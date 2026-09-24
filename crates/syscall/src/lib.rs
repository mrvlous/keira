// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! System call routing, exception dispatching, safe user pointer copying, and TSS configuration.

#![no_std]

pub mod dispatcher;
pub mod exception;
pub mod table;
pub mod tss;
pub mod user_copy;

pub use dispatcher::{syscall_dispatcher, validate_fd};
pub use exception::{exception_dispatcher, ExceptionStackFrame};
pub use table::*;
pub use tss::{get_boot_kernel_stack, init_user_mode, set_kernel_stack, TaskStateSegment, TSS};
pub use user_copy::{
    copy_from_user, copy_to_user, copy_val_from_user, copy_val_to_user, errno_to_ret,
    read_user_string, read_user_string_bounded, ret_to_errno, validate_user_ptr, EACCES, EAGAIN,
    EBADF, ECHILD, EEXIST, EFAULT, EINTR, EINVAL, EIO, EMFILE, ENOENT, ENOMEM, ENOSYS, EPERM,
    ESRCH, USER_MAX_ADDR, USER_MIN_ADDR,
};

#[cfg(test)]
mod tests;
