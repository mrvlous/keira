// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! POSIX-compatible numeric error codes and translation helpers.
//!
//! Bridges high-level `KernelError` representations with numeric system call
//! return values expected by standard userland libraries and runtimes.

use super::code::KernelError;

/// Operation not permitted.
pub const EPERM: i32 = 1;
/// No such file or directory.
pub const ENOENT: i32 = 2;
/// I/O error.
pub const EIO: i32 = 5;
/// Bad file descriptor.
pub const EBADF: i32 = 9;
/// Out of memory.
pub const ENOMEM: i32 = 12;
/// Permission denied.
pub const EACCES: i32 = 13;
/// Bad address.
pub const EFAULT: i32 = 14;
/// Device or resource busy.
pub const EBUSY: i32 = 16;
/// File exists.
pub const EEXIST: i32 = 17;
/// Invalid argument.
pub const EINVAL: i32 = 22;
/// Operation not supported on socket or transport.
pub const ENOTSUP: i32 = 95;
/// Connection timed out.
pub const ETIMEDOUT: i32 = 110;

/// Converts a high-level `KernelError` into a POSIX numeric error code.
pub const fn error_to_errno(err: KernelError) -> i32 {
    match err {
        KernelError::Generic(_) => EINVAL,
        KernelError::OutOfMemory => ENOMEM,
        KernelError::InvalidArgument => EINVAL,
        KernelError::PermissionDenied => EACCES,
        KernelError::NotFound => ENOENT,
        KernelError::DeviceBusy => EBUSY,
        KernelError::TimedOut => ETIMEDOUT,
        KernelError::NotSupported => ENOTSUP,
        KernelError::IoError => EIO,
        KernelError::AlreadyExists => EEXIST,
        KernelError::BadDescriptor => EBADF,
        KernelError::Fault => EFAULT,
    }
}

/// Converts a POSIX numeric error code into a negative system call return value.
#[inline(always)]
pub const fn errno_to_ret(errno: i32) -> i64 {
    -(errno as i64)
}
