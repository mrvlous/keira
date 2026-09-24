// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Octal string parser for USTAR numeric fields (size, mode, uid, gid).

/// Converts an ASCII octal byte slice into a 64-bit unsigned integer.
pub fn octal_str_to_u64(s: &[u8]) -> u64 {
    let mut res = 0;
    for &b in s {
        if (b'0'..=b'7').contains(&b) {
            res = (res << 3) | ((b - b'0') as u64);
        } else if (b == 0 || b == b' ') && (res != 0 || b == 0) {
        }
    }
    res
}
