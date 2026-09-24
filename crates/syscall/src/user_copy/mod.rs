// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Centralized safe user space memory validation, range checking, and copying primitives.

pub mod copy;
pub mod errno;
pub mod string;
pub mod validate;

#[cfg(test)]
mod tests;

pub use copy::{copy_from_user, copy_to_user, copy_val_from_user, copy_val_to_user};
pub use errno::{
    errno_to_ret, ret_to_errno, EACCES, EAGAIN, EBADF, ECHILD, EEXIST, EFAULT, EINTR, EINVAL, EIO,
    EMFILE, ENOENT, ENOMEM, ENOSYS, EPERM, ESRCH,
};
pub use string::{read_user_string, read_user_string_bounded};
pub use validate::{validate_user_ptr, USER_MAX_ADDR, USER_MIN_ADDR};
