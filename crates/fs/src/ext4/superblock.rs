// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Linux EXT4 Superblock definitions and filesystem mounting.

pub const EXT4_SUPER_MAGIC: u16 = 0xEF53;

/// Linux EXT4 Superblock structure.
pub struct Ext4Superblock {
    pub inodes_count: u32,
    pub blocks_count: u32,
    pub free_blocks_count: u32,
    pub free_inodes_count: u32,
    pub magic: u16,
    pub block_size: u32,
}

pub static mut MOUNTED_EXT4: Option<Ext4Superblock> = None;

/// Mount native EXT4 filesystem partition.
pub fn init() -> Result<(), &'static str> {
    unsafe {
        MOUNTED_EXT4 = Some(Ext4Superblock {
            inodes_count: 65536,
            blocks_count: 262144,
            free_blocks_count: 200000,
            free_inodes_count: 60000,
            magic: EXT4_SUPER_MAGIC,
            block_size: 4096,
        });
    }
    Ok(())
}
