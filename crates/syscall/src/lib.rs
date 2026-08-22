// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

#![no_std]

//! System call routing, exception dispatching, safe user pointer copying, and TSS configuration.

pub mod dispatcher;
pub mod exception;
pub mod table;
pub mod tss;
pub mod user_copy;

pub use dispatcher::{syscall_dispatcher, validate_fd};
pub use exception::{exception_dispatcher, ExceptionStackFrame};
pub use table::*;
pub use tss::{init_user_mode, set_kernel_stack, TaskStateSegment, TSS};
pub use user_copy::{
    copy_from_user, copy_to_user, errno_to_ret, read_user_string, validate_user_ptr, EACCES,
    EAGAIN, EBADF, ECHILD, EFAULT, EINTR, EINVAL, EIO, ENOENT, ENOMEM, ENOSYS, EPERM,
    USER_MAX_ADDR, USER_MIN_ADDR,
};
