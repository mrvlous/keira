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

//! POSIX file descriptor polling system call handler (`SYS_POLL`).

#![allow(static_mut_refs)]

use keira_task::scheduler::{CURRENT_TASK_IDX, TASKS};
use keira_task::types::MAX_FDS;

use crate::user_copy::{
    copy_from_user, copy_to_user, errno_to_ret, validate_user_ptr, EFAULT, EINVAL,
};

/// Data is ready to be read from file descriptor.
pub const POLLIN: i16 = 0x0001;

/// Urgent or high-priority data available to read.
pub const POLLPRI: i16 = 0x0002;

/// Writing data will not block.
pub const POLLOUT: i16 = 0x0004;

/// Error condition occurred on the descriptor.
pub const POLLERR: i16 = 0x0008;

/// Hangup occurred on the descriptor (e.g. pipe peer closed).
pub const POLLHUP: i16 = 0x0010;

/// Invalid request: file descriptor is not open.
pub const POLLNVAL: i16 = 0x0020;

/// Maximum number of file descriptors monitored in a single poll invocation.
pub const MAX_POLL_FDS: usize = 64;

/// Standard POSIX pollfd structure layout.
#[repr(C)]
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct PollFd {
    pub fd: i32,
    pub events: i16,
    pub revents: i16,
}

/// System-wide I/O multiplexing telemetry metrics.
#[derive(Copy, Clone, Debug, Default)]
pub struct PollTelemetry {
    pub total_polls: u64,
    pub total_ready_events: u64,
    pub total_timeouts: u64,
    pub total_hup_events: u64,
    pub total_nval_events: u64,
}

/// Global telemetry state for poll operations.
pub static mut POLL_TELEMETRY: PollTelemetry = PollTelemetry {
    total_polls: 0,
    total_ready_events: 0,
    total_timeouts: 0,
    total_hup_events: 0,
    total_nval_events: 0,
};

/// Retrieve snapshot of global polling telemetry.
pub fn get_poll_telemetry() -> PollTelemetry {
    unsafe { POLL_TELEMETRY }
}

/// Reset global polling telemetry (used during test harness reset).
pub fn poll_telemetry_reset() {
    unsafe {
        POLL_TELEMETRY = PollTelemetry::default();
    }
}

/// Syscall 106: Synchronous I/O multiplexing across multiple file descriptors.
///
/// Implements POSIX `poll(struct pollfd *fds, nfds_t nfds, int timeout)`.
pub fn handle_poll(fds_ptr: u64, nfds: u64, timeout: u64) -> u64 {
    let timeout_ms = timeout as i32;

    if nfds > (MAX_POLL_FDS as u64) {
        return errno_to_ret(EINVAL);
    }

    if nfds == 0 {
        if timeout_ms > 0 {
            #[cfg(not(test))]
            if keira_arch::timers::hpet::is_initialized() {
                keira_arch::timers::hpet::delay_nanos((timeout_ms as u64) * 1_000_000);
            }
        }
        return 0;
    }

    if fds_ptr == 0 {
        return errno_to_ret(EFAULT);
    }

    let align = core::mem::align_of::<PollFd>() as u64;
    if !fds_ptr.is_multiple_of(align) {
        return errno_to_ret(EINVAL);
    }

    let byte_count = (nfds as usize) * core::mem::size_of::<PollFd>();
    if let Err(e) = unsafe { validate_user_ptr(fds_ptr, byte_count as u64, true) } {
        return errno_to_ret(e);
    }

    let mut kernel_fds = [PollFd::default(); MAX_POLL_FDS];
    let num_fds = nfds as usize;

    let user_slice = unsafe {
        core::slice::from_raw_parts_mut(
            kernel_fds.as_mut_ptr() as *mut u8,
            num_fds * core::mem::size_of::<PollFd>(),
        )
    };

    if let Err(e) = unsafe { copy_from_user(user_slice, fds_ptr) } {
        return errno_to_ret(e);
    }

    unsafe {
        POLL_TELEMETRY.total_polls = POLL_TELEMETRY.total_polls.saturating_add(1);
    }

    let mut ready_count: usize;
    let mut iterations = 0usize;
    let max_iterations = if timeout_ms <= 0 {
        1
    } else {
        (timeout_ms as usize).max(1)
    };

    loop {
        ready_count = 0;

        for item in kernel_fds[..num_fds].iter_mut() {
            item.revents = 0;

            if item.fd < 0 {
                continue;
            }

            if (item.fd as usize) >= MAX_FDS {
                item.revents = POLLNVAL;
                ready_count += 1;
                unsafe {
                    POLL_TELEMETRY.total_nval_events =
                        POLL_TELEMETRY.total_nval_events.saturating_add(1);
                }
                continue;
            }

            let desc_opt = unsafe {
                let task_idx = CURRENT_TASK_IDX;
                match TASKS[task_idx] {
                    Some(ref t) => {
                        let d = t.fds[item.fd as usize];
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
                    let mut rev = 0i16;

                    if desc.is_pipe {
                        let (readable, writable, hangup, error) = unsafe {
                            keira_ipc::pipe::query_pipe_readiness(desc.pipe_id, desc.pipe_write)
                        };
                        if (item.events & POLLIN) != 0 && readable {
                            rev |= POLLIN;
                        }
                        if (item.events & POLLOUT) != 0 && writable {
                            rev |= POLLOUT;
                        }
                        if hangup {
                            rev |= POLLHUP;
                            unsafe {
                                POLL_TELEMETRY.total_hup_events =
                                    POLL_TELEMETRY.total_hup_events.saturating_add(1);
                            }
                        }
                        if error {
                            rev |= POLLERR;
                        }
                    } else if desc.is_socket {
                        let sock_id = desc.socket_id as u64;
                        if (item.events & POLLIN) != 0
                            && unsafe { keira_net::socket::socket_is_readable(sock_id) }
                        {
                            rev |= POLLIN;
                        }
                        if (item.events & POLLOUT) != 0
                            && unsafe { keira_net::socket::socket_is_writable(sock_id) }
                        {
                            rev |= POLLOUT;
                        }
                    } else {
                        if (item.events & POLLIN) != 0 {
                            rev |= POLLIN;
                        }
                        if (item.events & POLLOUT) != 0 && desc.write_mode {
                            rev |= POLLOUT;
                        }
                    }

                    item.revents = rev;
                    if rev != 0 {
                        ready_count += 1;
                    }
                }
                None => {
                    item.revents = POLLNVAL;
                    ready_count += 1;
                    unsafe {
                        POLL_TELEMETRY.total_nval_events =
                            POLL_TELEMETRY.total_nval_events.saturating_add(1);
                    }
                }
            }
        }

        if ready_count > 0 || timeout_ms == 0 {
            break;
        }

        iterations += 1;
        if timeout_ms > 0 && iterations >= max_iterations {
            unsafe {
                POLL_TELEMETRY.total_timeouts = POLL_TELEMETRY.total_timeouts.saturating_add(1);
            }
            break;
        }

        #[cfg(not(test))]
        if keira_arch::timers::hpet::is_initialized() {
            keira_arch::timers::hpet::delay_nanos(1_000_000);
        } else {
            core::hint::spin_loop();
        }

        #[cfg(test)]
        break;
    }

    unsafe {
        POLL_TELEMETRY.total_ready_events = POLL_TELEMETRY
            .total_ready_events
            .saturating_add(ready_count as u64);
    }

    let out_slice = unsafe {
        core::slice::from_raw_parts(
            kernel_fds.as_ptr() as *const u8,
            num_fds * core::mem::size_of::<PollFd>(),
        )
    };

    if let Err(e) = unsafe { copy_to_user(fds_ptr, out_slice) } {
        return errno_to_ret(e);
    }

    ready_count as u64
}
