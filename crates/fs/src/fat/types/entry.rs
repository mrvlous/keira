// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Standard 32-byte FAT directory entry structure definition.

/// Standard 32-byte 8.3 FAT directory entry structure.
#[repr(C, packed)]
#[derive(Copy, Clone, Debug)]
pub struct DirectoryEntry {
    /// 8-byte filename and 3-byte extension in uppercase ASCII padded with spaces.
    pub name: [u8; 11],
    /// File attributes bitfield (Read-only: 0x01, Hidden: 0x02, System: 0x04, Volume ID: 0x08, Directory: 0x10, Archive: 0x20).
    pub attr: u8,
    /// Reserved for use by Windows NT.
    pub nt_res: u8,
    /// Millisecond stamp at file creation time (tenths of a second).
    pub crt_time_tenth: u8,
    /// File creation time.
    pub crt_time: u16,
    /// File creation date.
    pub crt_date: u16,
    /// Last access date.
    pub lst_acc_date: u16,
    /// High 16 bits of entry's first cluster number (FAT32 only, 0 in FAT16).
    pub first_cluster_hi: u16,
    /// Last modification time.
    pub wrt_time: u16,
    /// Last modification date.
    pub wrt_date: u16,
    /// Low 16 bits of entry's first cluster number.
    pub first_cluster_lo: u16,
    /// Size of the file in bytes (0 for directories).
    pub file_size: u32,
}
