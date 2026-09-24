// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Linux EXT4 directory record structures and binary block serialization.

pub const EXT4_FT_UNKNOWN: u8 = 0;
pub const EXT4_FT_REG_FILE: u8 = 1;
pub const EXT4_FT_DIR: u8 = 2;
pub const EXT4_FT_CHRDEV: u8 = 3;
pub const EXT4_FT_BLKDEV: u8 = 4;
pub const EXT4_FT_FIFO: u8 = 5;
pub const EXT4_FT_SOCK: u8 = 6;
pub const EXT4_FT_SYMLINK: u8 = 7;

/// Directory entry in EXT4 linear directory block.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Ext4DirEntry {
    pub inode: u32,
    pub rec_len: u16,
    pub name_len: u8,
    pub file_type: u8,
    pub name: [u8; 32],
}

impl Default for Ext4DirEntry {
    fn default() -> Self {
        Self {
            inode: 0,
            rec_len: 0,
            name_len: 0,
            file_type: EXT4_FT_UNKNOWN,
            name: [0; 32],
        }
    }
}

impl Ext4DirEntry {
    /// Creates a new directory entry from fundamental components.
    pub const fn new(inode: u32, file_type: u8, name_bytes: &'static [u8], rec_len: u16) -> Self {
        let mut name = [0u8; 32];
        let mut i = 0;
        while i < name_bytes.len() && i < 32 {
            name[i] = name_bytes[i];
            i += 1;
        }
        Self {
            inode,
            rec_len,
            name_len: name_bytes.len() as u8,
            file_type,
            name,
        }
    }

    /// Retrieves the entry name as an immutable UTF-8 string slice.
    pub fn name_str(&self) -> &str {
        let len = (self.name_len as usize).min(self.name.len());
        core::str::from_utf8(&self.name[..len]).unwrap_or("unknown")
    }

    /// Returns human-readable string description of the file type.
    pub fn file_type_str(&self) -> &'static str {
        match self.file_type {
            EXT4_FT_REG_FILE => "file",
            EXT4_FT_DIR => "dir",
            EXT4_FT_CHRDEV => "chardev",
            EXT4_FT_BLKDEV => "blkdev",
            EXT4_FT_FIFO => "fifo",
            EXT4_FT_SOCK => "socket",
            EXT4_FT_SYMLINK => "symlink",
            _ => "unknown",
        }
    }
}

/// Parses linear `ext4_dir_entry_2` binary records from an on-disk directory data block.
pub fn parse_dir_entries_from_block(block: &[u8], out_entries: &mut [Ext4DirEntry]) -> usize {
    let mut offset = 0;
    let mut count = 0;

    while offset + 8 <= block.len() && count < out_entries.len() {
        let inode = u32::from_le_bytes([
            block[offset],
            block[offset + 1],
            block[offset + 2],
            block[offset + 3],
        ]);
        let rec_len = u16::from_le_bytes([block[offset + 4], block[offset + 5]]);
        let name_len = block[offset + 6];
        let file_type = block[offset + 7];

        if rec_len < 8 || (offset + rec_len as usize) > block.len() {
            break;
        }

        if inode != 0 {
            let mut name = [0u8; 32];
            let copy_len = (name_len as usize).min(32).min(rec_len as usize - 8);
            if offset + 8 + copy_len <= block.len() {
                name[..copy_len].copy_from_slice(&block[offset + 8..offset + 8 + copy_len]);
            }

            out_entries[count] = Ext4DirEntry {
                inode,
                rec_len,
                name_len,
                file_type,
                name,
            };
            count += 1;
        }

        offset += rec_len as usize;
    }

    count
}

/// Serializes a directory entry into on-disk `ext4_dir_entry_2` format.
pub fn write_dir_entry_to_block(entry: &Ext4DirEntry, block: &mut [u8], offset: usize) -> usize {
    let rec_len = entry.rec_len as usize;
    if offset + rec_len > block.len() {
        return 0;
    }

    block[offset..offset + 4].copy_from_slice(&entry.inode.to_le_bytes());
    block[offset + 4..offset + 6].copy_from_slice(&entry.rec_len.to_le_bytes());
    block[offset + 6] = entry.name_len;
    block[offset + 7] = entry.file_type;

    let n_len = (entry.name_len as usize)
        .min(32)
        .min(rec_len.saturating_sub(8));
    block[offset + 8..offset + 8 + n_len].copy_from_slice(&entry.name[..n_len]);

    rec_len
}
