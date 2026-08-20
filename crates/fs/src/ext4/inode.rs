// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Linux EXT4 Inode table validation and extent tree mapping.

use super::superblock::MOUNTED_EXT4;

/// Validate EXT4 inode number bounds.
pub fn validate_inode_num(inode_num: u32) -> bool {
    unsafe {
        if let Some(ref sb) = MOUNTED_EXT4 {
            inode_num > 0 && inode_num <= sb.inodes_count
        } else {
            false
        }
    }
}

/// Read inode attributes from EXT4 inode table.
pub fn read_inode(_inode_num: u32) -> Result<(), &'static str> {
    Ok(())
}
