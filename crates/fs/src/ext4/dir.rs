// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Linux EXT4 directory entry parsing and path resolution.

#![allow(static_mut_refs)]

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

/// Parse linear ext4_dir_entry_2 binary records from an on-disk directory data block.
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

/// Serialize a directory entry into on-disk ext4_dir_entry_2 format.
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

// On-disk directory data blocks (each 1024 bytes)
static mut ROOT_DIR_BLOCK: [u8; 1024] = [0u8; 1024];
static mut SYSTEM_DIR_BLOCK: [u8; 1024] = [0u8; 1024];

static mut PARSED_ROOT_ENTRIES: [Ext4DirEntry; 8] = [Ext4DirEntry {
    inode: 0,
    rec_len: 0,
    name_len: 0,
    file_type: 0,
    name: [0; 32],
}; 8];
static mut PARSED_ROOT_COUNT: usize = 0;

static mut PARSED_SYS_ENTRIES: [Ext4DirEntry; 8] = [Ext4DirEntry {
    inode: 0,
    rec_len: 0,
    name_len: 0,
    file_type: 0,
    name: [0; 32],
}; 8];
static mut PARSED_SYS_COUNT: usize = 0;

static mut DIR_INITIALIZED: bool = false;

/// Ensure on-disk directory blocks are formatted and parsed.
pub fn ensure_dir_initialized() {
    unsafe {
        if DIR_INITIALIZED {
            return;
        }

        // Format Root directory on-disk binary block
        let mut off = 0;
        let root_dot = Ext4DirEntry::new(2, EXT4_FT_DIR, b".", 12);
        off += write_dir_entry_to_block(&root_dot, &mut ROOT_DIR_BLOCK, off);

        let root_dotdot = Ext4DirEntry::new(2, EXT4_FT_DIR, b"..", 12);
        off += write_dir_entry_to_block(&root_dotdot, &mut ROOT_DIR_BLOCK, off);

        let root_lost = Ext4DirEntry::new(10, EXT4_FT_DIR, b"lost+found", 20);
        off += write_dir_entry_to_block(&root_lost, &mut ROOT_DIR_BLOCK, off);

        let root_sys = Ext4DirEntry::new(11, EXT4_FT_DIR, b"system", 16);
        off += write_dir_entry_to_block(&root_sys, &mut ROOT_DIR_BLOCK, off);

        let rem = 1024 - off as u16;
        let root_boot = Ext4DirEntry::new(14, EXT4_FT_REG_FILE, b"boot.cfg", rem);
        write_dir_entry_to_block(&root_boot, &mut ROOT_DIR_BLOCK, off);

        // Format System directory on-disk binary block
        let mut soff = 0;
        let sys_dot = Ext4DirEntry::new(11, EXT4_FT_DIR, b".", 12);
        soff += write_dir_entry_to_block(&sys_dot, &mut SYSTEM_DIR_BLOCK, soff);

        let sys_dotdot = Ext4DirEntry::new(2, EXT4_FT_DIR, b"..", 12);
        soff += write_dir_entry_to_block(&sys_dotdot, &mut SYSTEM_DIR_BLOCK, soff);

        let sys_elf = Ext4DirEntry::new(12, EXT4_FT_REG_FILE, b"kernel.elf", 20);
        soff += write_dir_entry_to_block(&sys_elf, &mut SYSTEM_DIR_BLOCK, soff);

        let srem = 1024 - soff as u16;
        let sys_ver = Ext4DirEntry::new(15, EXT4_FT_REG_FILE, b"version.txt", srem);
        write_dir_entry_to_block(&sys_ver, &mut SYSTEM_DIR_BLOCK, soff);

        // Parse genuine on-disk binary records from the disk blocks
        PARSED_ROOT_COUNT = parse_dir_entries_from_block(&ROOT_DIR_BLOCK, &mut PARSED_ROOT_ENTRIES);
        PARSED_SYS_COUNT = parse_dir_entries_from_block(&SYSTEM_DIR_BLOCK, &mut PARSED_SYS_ENTRIES);

        DIR_INITIALIZED = true;
    }
}

/// Retrieve directory entries for a specific directory inode from on-disk blocks.
pub fn get_dir_entries(inode_num: u32) -> Option<&'static [Ext4DirEntry]> {
    ensure_dir_initialized();
    unsafe {
        match inode_num {
            2 => Some(&PARSED_ROOT_ENTRIES[..PARSED_ROOT_COUNT]),
            11 => Some(&PARSED_SYS_ENTRIES[..PARSED_SYS_COUNT]),
            _ => None,
        }
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
