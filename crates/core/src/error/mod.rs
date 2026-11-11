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

//! Unified error subsystem and status code translations.
//!
//! Exposes kernel error variants, result aliases and POSIX errno mappings
//! for hardware drivers and system call handling paths.

pub mod code;
pub mod errno;

pub use code::{KernelError, Result};
pub use errno::{
    errno_to_ret, error_to_errno, EACCES, EBADF, EBUSY, EEXIST, EFAULT, EINVAL, EIO, ENOENT,
    ENOMEM, ENOTSUP, EPERM, ETIMEDOUT,
};
