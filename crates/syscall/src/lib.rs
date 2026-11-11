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

//! System call routing, exception dispatching, safe user pointer copying and TSS configuration.

#![no_std]

pub mod dispatcher;
pub mod exception;
pub mod table;
pub mod tss;
pub mod user_copy;

pub use dispatcher::{
    get_poll_telemetry, get_select_telemetry, handle_select, poll_telemetry_reset,
    select_telemetry_reset, syscall_dispatcher, validate_fd, FdSet, PollFd, PollTelemetry,
    SelectTelemetry, TimeVal, FD_SETSIZE, POLLERR, POLLHUP, POLLIN, POLLNVAL, POLLOUT, POLLPRI,
};
pub use exception::{
    exception_dispatcher, get_cpu_exception_count, ExceptionStackFrame, TOTAL_CPU_EXCEPTIONS,
};
pub use table::*;
pub use tss::{get_boot_kernel_stack, init_user_mode, set_kernel_stack, TaskStateSegment, TSS};
pub use user_copy::{
    copy_from_user, copy_to_user, copy_val_from_user, copy_val_to_user, errno_to_ret,
    read_user_string, read_user_string_bounded, ret_to_errno, validate_user_ptr, EACCES, EAGAIN,
    EBADF, ECHILD, EEXIST, EFAULT, EINTR, EINVAL, EIO, EMFILE, ENOENT, ENOMEM, ENOSYS, EPERM,
    EPIPE, ESRCH, USER_MAX_ADDR, USER_MIN_ADDR,
};

#[cfg(test)]
mod tests;
