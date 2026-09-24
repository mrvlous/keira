// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Error number translation routines between POSIX and unsigned system call registers.

/// Converts a signed POSIX error number into an unsigned syscall return value (`-err as u64`).
#[inline]
pub fn errno_to_ret(err: i64) -> u64 {
    (-err) as u64
}

/// Converts an unsigned syscall return value back into a signed POSIX error number if in error range.
#[inline]
pub fn ret_to_errno(ret: u64) -> Option<i64> {
    let signed = ret as i64;
    if signed < 0 && signed >= -4095 {
        Some(-signed)
    } else {
        None
    }
}
