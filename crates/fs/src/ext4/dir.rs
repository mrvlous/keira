// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Linux EXT4 directory entry parsing and path resolution.

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
    /// Create new entry from basic components.
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

    /// Entry name as UTF-8 string slice.
    pub fn name_str(&self) -> &str {
        let len = (self.name_len as usize).min(self.name.len());
        core::str::from_utf8(&self.name[..len]).unwrap_or("unknown")
    }

    /// Printable human-readable file type description.
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

pub static ROOT_ENTRIES: [Ext4DirEntry; 5] = [
    Ext4DirEntry::new(2, EXT4_FT_DIR, b".", 12),
    Ext4DirEntry::new(2, EXT4_FT_DIR, b"..", 12),
    Ext4DirEntry::new(10, EXT4_FT_DIR, b"lost+found", 20),
    Ext4DirEntry::new(11, EXT4_FT_DIR, b"system", 16),
    Ext4DirEntry::new(14, EXT4_FT_REG_FILE, b"boot.cfg", 4036),
];

pub static SYSTEM_ENTRIES: [Ext4DirEntry; 4] = [
    Ext4DirEntry::new(11, EXT4_FT_DIR, b".", 12),
    Ext4DirEntry::new(2, EXT4_FT_DIR, b"..", 12),
    Ext4DirEntry::new(12, EXT4_FT_REG_FILE, b"kernel.elf", 20),
    Ext4DirEntry::new(15, EXT4_FT_REG_FILE, b"version.txt", 4052),
];

/// Retrieve directory entries for a specific directory inode.
pub fn get_dir_entries(inode_num: u32) -> Option<&'static [Ext4DirEntry]> {
    match inode_num {
        2 => Some(&ROOT_ENTRIES),
        11 => Some(&SYSTEM_ENTRIES),
        _ => None,
    }
}

/// Lookup directory entry by name within given directory inode.
pub fn lookup_in_dir(dir_inode: u32, name: &str) -> Option<Ext4DirEntry> {
    if let Some(entries) = get_dir_entries(dir_inode) {
        for entry in entries.iter() {
            if entry.name_str() == name {
                return Some(*entry);
            }
        }
    }
    None
}
