// SPDX-License-Identifier: GPL-2.0-only
/*
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra <mrvlous@proton.me>
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 */

//! In-memory circular syslog dmesg diagnostic ring buffer storage.
//!
//! Stores formatted kernel diagnostic messages in a statically allocated memory
//! buffer accessible across panic handlers, syscalls and userland diagnostic tools.

use core::fmt::Write;

/// Capacity of the static in-memory kernel log ring buffer in bytes (32 KiB).
pub const KLOG_BUFFER_SIZE: usize = 32768;

/// Primary kernel log ring buffer storage for diagnostic telemetry.
///
/// # Safety Considerations
///
/// Direct access to this static buffer requires external mutual exclusion or single-core
/// early boot contexts to prevent data race conditions.
pub static mut KLOG_RING_BUFFER: [u8; KLOG_BUFFER_SIZE] = [0u8; KLOG_BUFFER_SIZE];

/// Current write head offset within the kernel log ring buffer.
pub static mut KLOG_HEAD: usize = 0;

/// Total number of diagnostic bytes written to the kernel log ring buffer since boot.
pub static mut KLOG_TOTAL_WRITTEN: usize = 0;

/// Diagnostic log multiplexer sink callback type.
pub type LogSinkFn = fn(level: u8, tag: &str, formatted: &str);

/// Monotonic timestamp provider callback type returning elapsed microseconds.
pub type UptimeProviderFn = fn() -> u64;

/// Active logging output sink callback.
static mut LOG_SINK: Option<LogSinkFn> = None;

/// Active monotonic uptime microsecond provider.
static mut UPTIME_PROVIDER: Option<UptimeProviderFn> = None;

/// Registers the primary kernel log sink callback for serial and VGA multiplexing.
pub fn set_log_sink(sink: LogSinkFn) {
    unsafe {
        LOG_SINK = Some(sink);
    }
}

/// Registers the monotonic uptime provider callback for microsecond timestamps.
pub fn set_uptime_provider(provider: UptimeProviderFn) {
    unsafe {
        UPTIME_PROVIDER = Some(provider);
    }
}

/// Stack-allocated fixed buffer implementing `core::fmt::Write`.
pub struct FixedWriter<const N: usize> {
    buf: [u8; N],
    len: usize,
}

impl<const N: usize> FixedWriter<N> {
    /// Constructs a new empty `FixedWriter`.
    pub const fn new() -> Self {
        Self {
            buf: [0u8; N],
            len: 0,
        }
    }

    /// Returns the written contents as a string slice.
    pub fn as_str(&self) -> &str {
        core::str::from_utf8(&self.buf[..self.len]).unwrap_or("")
    }

    /// Returns the written contents as a byte slice.
    pub fn as_bytes(&self) -> &[u8] {
        &self.buf[..self.len]
    }

    /// Returns the number of bytes written.
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` if no bytes have been written.
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }
}

impl<const N: usize> Default for FixedWriter<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> Write for FixedWriter<N> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let bytes = s.as_bytes();
        let remaining = N.saturating_sub(self.len);
        let to_copy = bytes.len().min(remaining);
        self.buf[self.len..self.len + to_copy].copy_from_slice(&bytes[..to_copy]);
        self.len += to_copy;
        Ok(())
    }
}

/// Formats and appends a kernel diagnostic log message with timestamp and subsystem tag.
///
/// # Arguments
///
/// * `level` - Log severity level corresponding to `KERN_*` constants.
/// * `tag` - Subsystem or driver identifier tag (e.g. `pmm`, `vmm`, `e1000`).
/// * `msg` - Unformatted or pre-formatted message text slice.
pub fn klog_write(level: u8, tag: &str, msg: &str) {
    let mut line_buf = FixedWriter::<1024>::new();
    let us = if let Some(provider) = unsafe { UPTIME_PROVIDER } {
        provider()
    } else {
        0
    };
    let secs = us / 1_000_000;
    let micros = us % 1_000_000;

    let _ = core::fmt::write(
        &mut line_buf,
        format_args!("[ {:5}.{:06} ] {}: {}", secs, micros, tag, msg),
    );
    if !msg.ends_with('\n') {
        let _ = line_buf.write_str("\n");
    }

    let bytes = line_buf.as_bytes();
    unsafe {
        for &b in bytes {
            KLOG_RING_BUFFER[KLOG_HEAD] = b;
            KLOG_HEAD = (KLOG_HEAD + 1) % KLOG_BUFFER_SIZE;
        }
        KLOG_TOTAL_WRITTEN = KLOG_TOTAL_WRITTEN.saturating_add(bytes.len());
    }

    if let Some(sink) = unsafe { LOG_SINK } {
        sink(level, tag, line_buf.as_str());
    }
}

/// Helper for logging formatted arguments directly without heap allocations.
pub fn klog_fmt(level: u8, tag: &str, args: core::fmt::Arguments) {
    let mut msg_buf = FixedWriter::<512>::new();
    let _ = core::fmt::write(&mut msg_buf, args);
    klog_write(level, tag, msg_buf.as_str());
}

/// Appends a diagnostic log message to the circular kernel syslog ring buffer.
///
/// Legacy compatibility helper defaulting subsystem tag to `"kernel"`.
pub fn klog(level: u8, msg: &str) {
    klog_write(level, "kernel", msg);
}

/// Returns the total number of diagnostic bytes written to the ring buffer since boot.
pub fn klog_total_written() -> usize {
    unsafe { KLOG_TOTAL_WRITTEN }
}

/// Iterates over active log records in chronological order invoking a caller callback.
///
/// # Arguments
///
/// * `max_len` - Maximum total bytes requested by caller.
/// * `callback` - Invoked with continuous slices of chronological log data.
pub fn klog_get_records<F>(max_len: usize, mut callback: F) -> usize
where
    F: FnMut(&[u8]) -> Result<(), ()>,
{
    if max_len == 0 {
        return 0;
    }

    unsafe {
        let total = KLOG_TOTAL_WRITTEN;
        if total == 0 {
            return 0;
        }

        let available = total.min(KLOG_BUFFER_SIZE);
        let copy_len = available.min(max_len);

        if total <= KLOG_BUFFER_SIZE {
            let start = available - copy_len;
            let slice = &KLOG_RING_BUFFER[start..available];
            if callback(slice).is_ok() {
                copy_len
            } else {
                0
            }
        } else {
            let start = (KLOG_HEAD + KLOG_BUFFER_SIZE - copy_len) % KLOG_BUFFER_SIZE;
            let first_len = (KLOG_BUFFER_SIZE - start).min(copy_len);
            let first_slice = &KLOG_RING_BUFFER[start..start + first_len];
            if callback(first_slice).is_err() {
                return 0;
            }

            let second_len = copy_len - first_len;
            if second_len > 0 {
                let second_slice = &KLOG_RING_BUFFER[0..second_len];
                if callback(second_slice).is_err() {
                    return first_len;
                }
            }
            copy_len
        }
    }
}

/// Reads chronological ring buffer records into a destination byte buffer.
///
/// # Arguments
///
/// * `dest` - Destination byte slice to receive log messages.
///
/// # Returns
///
/// Returns the number of bytes copied into `dest`.
pub fn klog_read_all(dest: &mut [u8]) -> usize {
    if dest.is_empty() {
        return 0;
    }
    let mut copied = 0;
    let dest_len = dest.len();
    klog_get_records(dest_len, |chunk| {
        let to_copy = chunk.len().min(dest_len.saturating_sub(copied));
        dest[copied..copied + to_copy].copy_from_slice(&chunk[..to_copy]);
        copied += to_copy;
        Ok(())
    });
    copied
}

/// Clears all stored messages and resets the kernel log ring buffer.
pub fn klog_clear() {
    unsafe {
        KLOG_HEAD = 0;
        KLOG_TOTAL_WRITTEN = 0;
        KLOG_RING_BUFFER = [0u8; KLOG_BUFFER_SIZE];
    }
}
