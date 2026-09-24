// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! In-memory circular syslog dmesg diagnostic ring buffer storage.
//!
//! Stores formatted kernel diagnostic messages in a statically allocated memory
//! buffer accessible across panic handlers and userland diagnostic tools.

/// Capacity of the static in-memory kernel log ring buffer in bytes.
pub const KLOG_BUFFER_SIZE: usize = 4096;

/// Primary kernel log ring buffer storage for diagnostic telemetry.
///
/// # Safety Considerations
///
/// Direct access to this static buffer requires external mutual exclusion or single-core
/// early boot contexts to prevent data race conditions.
pub static mut KLOG_RING_BUFFER: [u8; KLOG_BUFFER_SIZE] = [0u8; KLOG_BUFFER_SIZE];

/// Current write head offset within the kernel log ring buffer.
pub static mut KLOG_HEAD: usize = 0;

/// Appends a diagnostic log message to the circular kernel syslog ring buffer.
///
/// # Arguments
///
/// * `_level` - Log severity level corresponding to `KERN_*` constants.
/// * `msg` - Text message slice to append into the circular buffer.
///
/// # Safety
///
/// This function internally accesses mutable static variables `KLOG_RING_BUFFER` and
/// `KLOG_HEAD`. In multi-core runtime contexts, callers should ensure appropriate
/// synchronization or invoke via serialized logging macros.
pub fn klog(_level: u8, msg: &str) {
    // Write message bytes directly into the circular buffer, wrapping on overflow
    unsafe {
        let bytes = msg.as_bytes();
        for &b in bytes {
            KLOG_RING_BUFFER[KLOG_HEAD] = b;
            KLOG_HEAD = (KLOG_HEAD + 1) % KLOG_BUFFER_SIZE;
        }
    }
}
