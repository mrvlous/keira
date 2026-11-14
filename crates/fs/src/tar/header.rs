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

//! Tar archive header structures and octal number parsing.

pub mod octal {
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
}

pub mod types {
    #[repr(C)]
    #[derive(Clone, Copy, Debug)]
    pub struct UstarHeader {
        pub name: [u8; 100],
        pub mode: [u8; 8],
        pub uid: [u8; 8],
        pub gid: [u8; 8],
        pub size: [u8; 12],
        pub mtime: [u8; 12],
        pub chksum: [u8; 8],
        pub typeflag: u8,
        pub linkname: [u8; 100],
        pub magic: [u8; 6],
        pub version: [u8; 2],
        pub uname: [u8; 32],
        pub gname: [u8; 32],
        pub devmajor: [u8; 8],
        pub devminor: [u8; 8],
        pub prefix: [u8; 155],
        pub pad: [u8; 12],
    }
    
    pub const TYPEFLAG_REGULAR: u8 = b'0';
    pub const TYPEFLAG_REGULAR_ALT: u8 = 0;
    pub const TYPEFLAG_DIRECTORY: u8 = b'5';
}

pub use self::octal::*;
pub use self::types::*;
