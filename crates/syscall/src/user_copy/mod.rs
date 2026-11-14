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

//! Centralized safe user space memory validation, range checking and copying primitives.

pub mod buffer;
pub mod errno;
pub mod string;
pub mod validate;

#[cfg(test)]
mod tests;

pub use buffer as copy;
pub use buffer::{copy_from_user, copy_to_user, copy_val_from_user, copy_val_to_user};
pub use errno::{
    errno_to_ret, ret_to_errno, EACCES, EAFNOSUPPORT, EAGAIN, EBADF, EBUSY, ECHILD, EEXIST, EFAULT,
    EFBIG, EINTR, EINVAL, EIO, EISDIR, ELOOP, EMFILE, ENAMETOOLONG, ENFILE, ENODEV, ENOENT, ENOMEM,
    ENOSPC, ENOSYS, ENOTDIR, ENOTEMPTY, EPERM, EPIPE, EPROTONOSUPPORT, ESRCH,
};
pub use string::{read_user_string, read_user_string_bounded};
pub use validate::{validate_user_ptr, USER_MAX_ADDR, USER_MIN_ADDR};
