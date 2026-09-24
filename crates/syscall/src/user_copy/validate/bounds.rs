// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! User space virtual address boundaries and range check functions.

use crate::user_copy::errno::EFAULT;

pub const USER_MIN_ADDR: u64 = 0x10000;

#[cfg(target_arch = "x86_64")]
pub const USER_MAX_ADDR: u64 = 0x0000_7FFF_FFFF_FFFF;

#[cfg(target_arch = "x86")]
pub const USER_MAX_ADDR: u64 = 0xBFFF_FFFF;

/// Validates that `[ptr, ptr + len)` is strictly contained within user space virtual address limits.
#[inline]
pub fn check_user_bounds(ptr: u64, len: u64) -> Result<u64, i64> {
    if len == 0 {
        return Ok(ptr);
    }

    let end = match ptr.checked_add(len) {
        Some(e) => e,
        None => return Err(EFAULT),
    };

    if ptr < USER_MIN_ADDR || end > USER_MAX_ADDR {
        return Err(EFAULT);
    }

    Ok(end)
}
