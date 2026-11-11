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
