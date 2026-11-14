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

//! Standard POSIX error codes and register translation routines.

pub const EPERM: i64 = 1;
pub const ENOENT: i64 = 2;
pub const ESRCH: i64 = 3;
pub const EINTR: i64 = 4;
pub const EIO: i64 = 5;
pub const EBADF: i64 = 9;
pub const ECHILD: i64 = 10;
pub const EAGAIN: i64 = 11;
pub const ENOMEM: i64 = 12;
pub const EACCES: i64 = 13;
pub const EFAULT: i64 = 14;
pub const EEXIST: i64 = 17;
pub const EINVAL: i64 = 22;
pub const EMFILE: i64 = 24;
pub const EFBIG: i64 = 27;
pub const ENOSPC: i64 = 28;
pub const ENOTDIR: i64 = 20;
pub const EISDIR: i64 = 21;
pub const ENOSYS: i64 = 38;
pub const ENOTEMPTY: i64 = 39;
pub const ENAMETOOLONG: i64 = 36;
pub const ELOOP: i64 = 40;
pub const EBUSY: i64 = 16;
pub const ENODEV: i64 = 19;
pub const EPIPE: i64 = 32;
pub const ENFILE: i64 = 23;
pub const EPROTONOSUPPORT: i64 = 93;
pub const EAFNOSUPPORT: i64 = 97;

/// Converts a signed POSIX error number into an unsigned syscall return value (`-err as u64`).
#[inline]
pub fn errno_to_ret(err: i64) -> u64 {
    (-err) as u64
}

/// Converts an unsigned syscall return value back into a signed POSIX error number if in error range.
#[inline]
pub fn ret_to_errno(ret: u64) -> Option<i64> {
    let signed = ret as i64;
    if (-4095..0).contains(&signed) {
        Some(-signed)
    } else {
        None
    }
}
