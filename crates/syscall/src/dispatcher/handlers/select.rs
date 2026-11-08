// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! POSIX synchronous I/O bitmask multiplexing handler (`SYS_SELECT`).

#![allow(static_mut_refs)]

use keira_task::scheduler::{CURRENT_TASK_IDX, TASKS};
use keira_task::types::MAX_FDS;

use crate::user_copy::{
    copy_from_user, copy_to_user, errno_to_ret, validate_user_ptr, EBADF, EINVAL,
};

/// Maximum file descriptor capacity representable by standard POSIX `fd_set`.
pub const FD_SETSIZE: usize = 1024;

/// Number of 64-bit words required to represent 1024 file descriptors.
pub const FD_SET_WORDS: usize = 16;

/// Standard POSIX `fd_set` bitmask layout (1024 bits = 128 bytes).
#[repr(C)]
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct FdSet {
    pub bits: [u64; FD_SET_WORDS],
}

impl FdSet {
    /// Returns true if the specified file descriptor bit is set.
    #[inline]
    pub fn is_set(&self, fd: usize) -> bool {
        if fd >= FD_SETSIZE {
            return false;
        }
        let word = fd / 64;
        let bit = fd % 64;
        (self.bits[word] & (1u64 << bit)) != 0
    }

    /// Sets the specified file descriptor bit in the mask.
    #[inline]
    pub fn set(&mut self, fd: usize) {
        if fd < FD_SETSIZE {
            let word = fd / 64;
            let bit = fd % 64;
            self.bits[word] |= 1u64 << bit;
        }
    }

    /// Clears the specified file descriptor bit in the mask.
    #[inline]
    pub fn clear(&mut self, fd: usize) {
        if fd < FD_SETSIZE {
            let word = fd / 64;
            let bit = fd % 64;
            self.bits[word] &= !(1u64 << bit);
        }
    }

    /// Clears all bits in the mask.
    #[inline]
    pub fn zero(&mut self) {
        self.bits = [0u64; FD_SET_WORDS];
    }
}

/// Standard POSIX `timeval` representation in seconds and microseconds.
#[repr(C)]
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct TimeVal {
    pub tv_sec: i64,
    pub tv_usec: i64,
}

/// Global telemetry metrics for `SYS_SELECT` operations.
#[derive(Copy, Clone, Debug, Default)]
pub struct SelectTelemetry {
    pub total_selects: u64,
    pub total_ready_events: u64,
    pub total_timeouts: u64,
    pub total_badf_events: u64,
}

/// System-wide telemetry state for select multiplexing.
pub static mut SELECT_TELEMETRY: SelectTelemetry = SelectTelemetry {
    total_selects: 0,
    total_ready_events: 0,
    total_timeouts: 0,
    total_badf_events: 0,
};

/// Retrieve snapshot of global select telemetry.
pub fn get_select_telemetry() -> SelectTelemetry {
    unsafe { SELECT_TELEMETRY }
}

/// Reset global select telemetry state (used during unit testing).
pub fn select_telemetry_reset() {
    unsafe {
        SELECT_TELEMETRY = SelectTelemetry::default();
    }
}

/// Syscall 107: Synchronous I/O multiplexing across descriptor bitmasks.
///
/// Implements POSIX `select(int nfds, fd_set *readfds, fd_set *writefds, fd_set *exceptfds, struct timeval *timeout)`.
pub fn handle_select(
    nfds: u64,
    readfds_ptr: u64,
    writefds_ptr: u64,
    exceptfds_ptr: u64,
    timeout_ptr: u64,
) -> u64 {
    let nfds_usize = nfds as usize;

    if nfds > (FD_SETSIZE as u64) {
        return errno_to_ret(EINVAL);
    }

    let mut timeout_ms = -1i64;
    let mut immediate = false;

    if timeout_ptr != 0 {
        let align = core::mem::align_of::<TimeVal>() as u64;
        if !timeout_ptr.is_multiple_of(align) {
            return errno_to_ret(EINVAL);
        }

        if let Err(e) =
            unsafe { validate_user_ptr(timeout_ptr, core::mem::size_of::<TimeVal>() as u64, false) }
        {
            return errno_to_ret(e);
        }

        let mut tv = TimeVal::default();
        let tv_slice = unsafe {
            core::slice::from_raw_parts_mut(
                &mut tv as *mut TimeVal as *mut u8,
                core::mem::size_of::<TimeVal>(),
            )
        };

        if let Err(e) = unsafe { copy_from_user(tv_slice, timeout_ptr) } {
            return errno_to_ret(e);
        }

        if tv.tv_sec < 0 || tv.tv_usec < 0 || tv.tv_usec >= 1_000_000 {
            return errno_to_ret(EINVAL);
        }

        if tv.tv_sec == 0 && tv.tv_usec == 0 {
            immediate = true;
            timeout_ms = 0;
        } else {
            let sec_ms = tv.tv_sec.saturating_mul(1000);
            let usec_ms = (tv.tv_usec + 999) / 1000;
            timeout_ms = sec_ms.saturating_add(usec_ms);
        }
    }

    // Pure sleep optimization: select(0, NULL, NULL, NULL, &tv)
    if nfds == 0 && readfds_ptr == 0 && writefds_ptr == 0 && exceptfds_ptr == 0 {
        if timeout_ms > 0 {
            #[cfg(not(test))]
            if keira_arch::timers::hpet::is_initialized() {
                keira_arch::timers::hpet::delay_nanos((timeout_ms as u64) * 1_000_000);
            }
        }
        return 0;
    }

    let fd_set_size = core::mem::size_of::<FdSet>() as u64;

    let mut in_read = FdSet::default();
    let mut in_write = FdSet::default();
    let mut in_except = FdSet::default();

    if readfds_ptr != 0 {
        if let Err(e) = unsafe { validate_user_ptr(readfds_ptr, fd_set_size, true) } {
            return errno_to_ret(e);
        }
        let slice = unsafe {
            core::slice::from_raw_parts_mut(
                &mut in_read as *mut FdSet as *mut u8,
                fd_set_size as usize,
            )
        };
        if let Err(e) = unsafe { copy_from_user(slice, readfds_ptr) } {
            return errno_to_ret(e);
        }
    }

    if writefds_ptr != 0 {
        if let Err(e) = unsafe { validate_user_ptr(writefds_ptr, fd_set_size, true) } {
            return errno_to_ret(e);
        }
        let slice = unsafe {
            core::slice::from_raw_parts_mut(
                &mut in_write as *mut FdSet as *mut u8,
                fd_set_size as usize,
            )
        };
        if let Err(e) = unsafe { copy_from_user(slice, writefds_ptr) } {
            return errno_to_ret(e);
        }
    }

    if exceptfds_ptr != 0 {
        if let Err(e) = unsafe { validate_user_ptr(exceptfds_ptr, fd_set_size, true) } {
            return errno_to_ret(e);
        }
        let slice = unsafe {
            core::slice::from_raw_parts_mut(
                &mut in_except as *mut FdSet as *mut u8,
                fd_set_size as usize,
            )
        };
        if let Err(e) = unsafe { copy_from_user(slice, exceptfds_ptr) } {
            return errno_to_ret(e);
        }
    }

    unsafe {
        SELECT_TELEMETRY.total_selects = SELECT_TELEMETRY.total_selects.saturating_add(1);
    }

    let mut out_read = FdSet::default();
    let mut out_write = FdSet::default();
    let mut out_except = FdSet::default();
    let mut ready_count: usize;

    let max_iterations = if immediate || timeout_ms <= 0 {
        1
    } else {
        (timeout_ms as usize).max(1)
    };
    let mut iterations = 0usize;

    loop {
        ready_count = 0;
        out_read.zero();
        out_write.zero();
        out_except.zero();

        let mut badf_found = false;

        for fd in 0..nfds_usize {
            let monitored_read = readfds_ptr != 0 && in_read.is_set(fd);
            let monitored_write = writefds_ptr != 0 && in_write.is_set(fd);
            let monitored_except = exceptfds_ptr != 0 && in_except.is_set(fd);

            if !monitored_read && !monitored_write && !monitored_except {
                continue;
            }

            if fd >= MAX_FDS {
                badf_found = true;
                break;
            }

            let desc_opt = unsafe {
                let task_idx = CURRENT_TASK_IDX;
                match TASKS[task_idx] {
                    Some(ref t) => {
                        let d = t.fds[fd];
                        if d.is_open {
                            Some(d)
                        } else {
                            None
                        }
                    }
                    None => None,
                }
            };

            match desc_opt {
                Some(desc) => {
                    if desc.is_pipe {
                        let (readable, writable, hangup, error) = unsafe {
                            keira_ipc::pipe::query_pipe_readiness(desc.pipe_id, desc.pipe_write)
                        };

                        if monitored_read && (readable || hangup) {
                            out_read.set(fd);
                            ready_count += 1;
                        }
                        if monitored_write && writable {
                            out_write.set(fd);
                            ready_count += 1;
                        }
                        if monitored_except && (error || hangup) {
                            out_except.set(fd);
                            ready_count += 1;
                        }
                    } else if desc.is_socket {
                        let sock_id = desc.socket_id as u64;
                        if monitored_read
                            && unsafe { keira_net::socket::socket_is_readable(sock_id) }
                        {
                            out_read.set(fd);
                            ready_count += 1;
                        }
                        if monitored_write
                            && unsafe { keira_net::socket::socket_is_writable(sock_id) }
                        {
                            out_write.set(fd);
                            ready_count += 1;
                        }
                    } else {
                        if monitored_read {
                            out_read.set(fd);
                            ready_count += 1;
                        }
                        if monitored_write && desc.write_mode {
                            out_write.set(fd);
                            ready_count += 1;
                        }
                    }
                }
                None => {
                    badf_found = true;
                    break;
                }
            }
        }

        if badf_found {
            unsafe {
                SELECT_TELEMETRY.total_badf_events =
                    SELECT_TELEMETRY.total_badf_events.saturating_add(1);
            }
            return errno_to_ret(EBADF);
        }

        if ready_count > 0 || immediate {
            break;
        }

        iterations += 1;
        if timeout_ms >= 0 && iterations >= max_iterations {
            unsafe {
                SELECT_TELEMETRY.total_timeouts = SELECT_TELEMETRY.total_timeouts.saturating_add(1);
            }
            break;
        }

        #[cfg(not(test))]
        if keira_arch::timers::hpet::is_initialized() {
            keira_arch::timers::hpet::delay_nanos(1_000_000);
        }
    }

    if readfds_ptr != 0 {
        let slice = unsafe {
            core::slice::from_raw_parts(
                &out_read as *const FdSet as *const u8,
                fd_set_size as usize,
            )
        };
        if let Err(e) = unsafe { copy_to_user(readfds_ptr, slice) } {
            return errno_to_ret(e);
        }
    }

    if writefds_ptr != 0 {
        let slice = unsafe {
            core::slice::from_raw_parts(
                &out_write as *const FdSet as *const u8,
                fd_set_size as usize,
            )
        };
        if let Err(e) = unsafe { copy_to_user(writefds_ptr, slice) } {
            return errno_to_ret(e);
        }
    }

    if exceptfds_ptr != 0 {
        let slice = unsafe {
            core::slice::from_raw_parts(
                &out_except as *const FdSet as *const u8,
                fd_set_size as usize,
            )
        };
        if let Err(e) = unsafe { copy_to_user(exceptfds_ptr, slice) } {
            return errno_to_ret(e);
        }
    }

    unsafe {
        SELECT_TELEMETRY.total_ready_events = SELECT_TELEMETRY
            .total_ready_events
            .saturating_add(ready_count as u64);
    }

    ready_count as u64
}
