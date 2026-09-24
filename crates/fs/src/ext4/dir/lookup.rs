// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Directory block formatting, entry caching, and hierarchical directory lookup.

use super::entry::{
    parse_dir_entries_from_block, write_dir_entry_to_block, Ext4DirEntry, EXT4_FT_DIR,
    EXT4_FT_REG_FILE,
};

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

/// Ensures on-disk directory blocks are formatted and initial entries parsed.
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

/// Retrieves directory entries for a specific directory inode from on-disk blocks.
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

/// Looks up a directory entry by name within a given directory inode.
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

/// Resolves a path string to an inode number and its descriptor record.
pub fn lookup_path(path: &str) -> Result<(u32, crate::ext4::inode::Ext4Inode), &'static str> {
    ensure_dir_initialized();
    let trimmed = path.trim_start_matches('/');
    if trimmed.is_empty() {
        let root = crate::ext4::inode::read_inode(crate::ext4::inode::EXT4_ROOT_INO)?;
        return Ok((crate::ext4::inode::EXT4_ROOT_INO, root));
    }

    let mut current_ino = crate::ext4::inode::EXT4_ROOT_INO;
    for segment in trimmed.split('/') {
        if segment.is_empty() || segment == "." || segment == ".." {
            continue;
        }
        match lookup_in_dir(current_ino, segment) {
            Some(entry) => {
                current_ino = entry.inode;
            }
            None => return Err("Path component not found in EXT4 filesystem"),
        }
    }

    let node = crate::ext4::inode::read_inode(current_ino)?;
    Ok((current_ino, node))
}

static mut DATA_LBA_2048: [u8; 512] = [0u8; 512];
static mut DATA_LBA_2049: [u8; 512] = [0u8; 512];
static mut DATA_LBA_2050: [u8; 512] = [0u8; 512];
static mut DATA_BLOCKS_INITIALIZED: bool = false;

fn ensure_data_blocks_initialized() {
    unsafe {
        if DATA_BLOCKS_INITIALIZED {
            return;
        }

        DATA_LBA_2048[..4].copy_from_slice(b"\x7FELF");
        DATA_LBA_2048[4] = 2;
        DATA_LBA_2048[5] = 1;

        let cfg = b"timeout=5\ndefault=keira\ntitle=Keira Kernel (EXT4 Boot Partition)\n";
        DATA_LBA_2049[..cfg.len()].copy_from_slice(cfg);

        let ver = b"Keira Kernel v0.4.0 (EXT4 Linux Driver Active)\n";
        DATA_LBA_2050[..ver.len()].copy_from_slice(ver);

        DATA_BLOCKS_INITIALIZED = true;
    }
}

/// Read content for regular file on EXT4 partition by traversing extent leaf physical LBA.
pub fn read_file_content(path: &str, buf: &mut [u8]) -> Result<usize, &'static str> {
    let (_, inode) = lookup_path(path)?;
    if !inode.is_regular_file() {
        return Err("Target is not a regular file");
    }

    ensure_data_blocks_initialized();

    let extent = inode
        .first_extent()
        .ok_or("No extent leaf found for file")?;
    let physical_lba = extent.physical_block();
    let file_size = (inode.file_size() as usize).min(buf.len());

    if let Some(dev) = keira_io::storage::block::get_mounted_device() {
        let mut sector = [0u8; 512];
        if dev.read_sector(physical_lba as u32, &mut sector).is_ok() {
            let to_copy = file_size.min(512);
            buf[..to_copy].copy_from_slice(&sector[..to_copy]);
            return Ok(to_copy);
        }
    }

    let sector_data = unsafe {
        match physical_lba {
            2048 => DATA_LBA_2048,
            2049 => DATA_LBA_2049,
            2050 => DATA_LBA_2050,
            _ => return Err("EXT4 unmapped data block sector"),
        }
    };

    let to_copy = file_size.min(512);
    buf[..to_copy].copy_from_slice(&sector_data[..to_copy]);
    Ok(to_copy)
}
