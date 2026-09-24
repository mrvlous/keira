// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! POSIX USTAR 512-byte header block structures and field offsets.

/// Standard 512-byte POSIX USTAR archive header block.
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

/// Regular file type flag ('0' or null byte).
pub const TYPEFLAG_REGULAR: u8 = b'0';
/// Alternative regular file type flag.
pub const TYPEFLAG_REGULAR_ALT: u8 = 0;
/// Directory type flag ('5').
pub const TYPEFLAG_DIRECTORY: u8 = b'5';
