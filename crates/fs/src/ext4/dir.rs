// SPDX-License-Identifier: GPL-2.0-only
//
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

//! Ext4 directory entry formatting and filename lookup.

pub mod entry {
    pub const EXT4_FT_UNKNOWN: u8 = 0;
    pub const EXT4_FT_REG_FILE: u8 = 1;
    pub const EXT4_FT_DIR: u8 = 2;
    pub const EXT4_FT_CHRDEV: u8 = 3;
    pub const EXT4_FT_BLKDEV: u8 = 4;
    pub const EXT4_FT_FIFO: u8 = 5;
    pub const EXT4_FT_SOCK: u8 = 6;
    pub const EXT4_FT_SYMLINK: u8 = 7;

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
        pub const fn new(
            inode: u32,
            file_type: u8,
            name_bytes: &'static [u8],
            rec_len: u16,
        ) -> Self {
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

    pub fn write_dir_entry_to_block(
        entry: &Ext4DirEntry,
        block: &mut [u8],
        offset: usize,
    ) -> usize {
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
}

pub mod lookup {
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

            let rem = 1024 - off as u16;
            let root_sys = Ext4DirEntry::new(11, EXT4_FT_DIR, b"system", rem);
            write_dir_entry_to_block(&root_sys, &mut ROOT_DIR_BLOCK, off);

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
            PARSED_ROOT_COUNT =
                parse_dir_entries_from_block(&ROOT_DIR_BLOCK, &mut PARSED_ROOT_ENTRIES);
            PARSED_SYS_COUNT =
                parse_dir_entries_from_block(&SYSTEM_DIR_BLOCK, &mut PARSED_SYS_ENTRIES);

            DIR_INITIALIZED = true;
        }
    }

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

    /// Computes aligned on-disk directory record size for `name_len`.
    pub const fn aligned_dir_entry_size(name_len: usize) -> usize {
        (8 + name_len + 3) & !3
    }

    /// Resolve one component `name` inside on-disk directory inode `dir_ino`.
    pub fn disk_lookup_in_dir(
        dir_ino: u32,
        name: &str,
    ) -> Result<Option<Ext4DirEntry>, &'static str> {
        let dir_inode = crate::ext4::inode::read_inode_disk(dir_ino)?;
        if !dir_inode.is_dir() {
            return Err("EXT4: Target inode is not a directory");
        }
        let num_blocks = dir_inode.file_size().div_ceil(4096).max(1) as u32;
        let mut blk = [0u8; 4096];

        for lblk in 0..num_blocks {
            let phys = match crate::ext4::inode::resolve_logical_block_disk(&dir_inode, lblk)? {
                Some(b) => b,
                None => continue,
            };
            crate::ext4::superblock::read_block_4k(phys, &mut blk)?;

            let mut off = 0usize;
            while off + 8 <= 4096 {
                let inode =
                    u32::from_le_bytes([blk[off], blk[off + 1], blk[off + 2], blk[off + 3]]);
                let rec_len = u16::from_le_bytes([blk[off + 4], blk[off + 5]]) as usize;
                let name_len = blk[off + 6] as usize;
                let file_type = blk[off + 7];
                if rec_len < 8 || off + rec_len > 4096 {
                    break;
                }
                if inode != 0 && off + 8 + name_len <= 4096 {
                    if let Ok(entry_name) = core::str::from_utf8(&blk[off + 8..off + 8 + name_len])
                    {
                        if entry_name == name {
                            let mut name_buf = [0u8; 32];
                            let copy_len = name_len.min(32);
                            name_buf[..copy_len].copy_from_slice(&blk[off + 8..off + 8 + copy_len]);
                            return Ok(Some(Ext4DirEntry {
                                inode,
                                rec_len: rec_len as u16,
                                name_len: copy_len as u8,
                                file_type,
                                name: name_buf,
                            }));
                        }
                    }
                }
                off += rec_len;
            }
        }
        Ok(None)
    }

    /// Resolve a path on the mounted EXT4 disk returning `(parent_ino, child_ino, child_inode)`.
    pub fn disk_lookup_path(
        path: &str,
    ) -> Result<(u32, u32, crate::ext4::inode::Ext4Inode), &'static str> {
        let trimmed = path.trim_matches('/');
        if trimmed.is_empty() {
            let root = crate::ext4::inode::read_inode_disk(crate::ext4::inode::EXT4_ROOT_INO)?;
            return Ok((
                crate::ext4::inode::EXT4_ROOT_INO,
                crate::ext4::inode::EXT4_ROOT_INO,
                root,
            ));
        }

        let mut parent_ino = crate::ext4::inode::EXT4_ROOT_INO;
        let mut current_ino = crate::ext4::inode::EXT4_ROOT_INO;
        for segment in trimmed.split('/') {
            if segment.is_empty() || segment == "." {
                continue;
            }
            if let Some(entry) = disk_lookup_in_dir(current_ino, segment)? {
                parent_ino = current_ino;
                current_ino = entry.inode;
            } else {
                return Err("Path component not found in EXT4 filesystem");
            }
        }

        let node = crate::ext4::inode::read_inode_disk(current_ino)?;
        Ok((parent_ino, current_ino, node))
    }

    /// Split parent directory path and final component filename cleanly.
    pub fn split_parent_and_name(path: &str) -> Result<(&str, &str), &'static str> {
        let trimmed = path.trim_matches('/');
        if trimmed.is_empty() {
            return Err("EXT4: Path has no leaf component");
        }
        if let Some(pos) = trimmed.rfind('/') {
            Ok((&trimmed[..pos], &trimmed[pos + 1..]))
        } else {
            Ok(("", trimmed))
        }
    }

    /// Insert a new directory entry into on-disk directory `parent_ino`.
    pub fn disk_add_dir_entry(
        parent_ino: u32,
        child_ino: u32,
        file_type: u8,
        name: &str,
    ) -> Result<(), &'static str> {
        let nbytes = name.as_bytes();
        if nbytes.is_empty() || nbytes.len() > 255 {
            return Err("EXT4: Invalid directory entry name length");
        }
        let needed = aligned_dir_entry_size(nbytes.len());
        let mut parent_inode = crate::ext4::inode::read_inode_disk(parent_ino)?;
        if !parent_inode.is_dir() {
            return Err("EXT4: Parent is not a directory");
        }
        let num_blocks = parent_inode.file_size().div_ceil(4096).max(1) as u32;
        let mut blk = [0u8; 4096];

        for lblk in 0..num_blocks {
            let phys = match crate::ext4::inode::resolve_logical_block_disk(&parent_inode, lblk)? {
                Some(b) => b,
                None => continue,
            };
            crate::ext4::superblock::read_block_4k(phys, &mut blk)?;

            let mut off = 0usize;
            while off + 8 <= 4096 {
                let inode =
                    u32::from_le_bytes([blk[off], blk[off + 1], blk[off + 2], blk[off + 3]]);
                let rec_len = u16::from_le_bytes([blk[off + 4], blk[off + 5]]) as usize;
                let cur_nlen = blk[off + 6] as usize;
                if rec_len < 8 || off + rec_len > 4096 {
                    break;
                }

                if inode == 0 && rec_len >= needed {
                    blk[off..off + 4].copy_from_slice(&child_ino.to_le_bytes());
                    blk[off + 4..off + 6].copy_from_slice(&(rec_len as u16).to_le_bytes());
                    blk[off + 6] = nbytes.len() as u8;
                    blk[off + 7] = file_type;
                    blk[off + 8..off + 8 + nbytes.len()].copy_from_slice(nbytes);
                    return crate::ext4::superblock::write_block_4k(phys, &blk);
                }

                let actual = aligned_dir_entry_size(cur_nlen);
                if inode != 0 && rec_len >= actual + needed {
                    let rem = rec_len - actual;
                    blk[off + 4..off + 6].copy_from_slice(&(actual as u16).to_le_bytes());
                    let new_off = off + actual;
                    blk[new_off..new_off + 4].copy_from_slice(&child_ino.to_le_bytes());
                    blk[new_off + 4..new_off + 6].copy_from_slice(&(rem as u16).to_le_bytes());
                    blk[new_off + 6] = nbytes.len() as u8;
                    blk[new_off + 7] = file_type;
                    blk[new_off + 8..new_off + 8 + nbytes.len()].copy_from_slice(nbytes);
                    return crate::ext4::superblock::write_block_4k(phys, &blk);
                }

                off += rec_len;
            }
        }

        // Append a brand new 4 KiB block to the directory extent tree
        let new_phys = crate::ext4::inode::alloc_and_map_block_disk(
            parent_ino,
            &mut parent_inode,
            num_blocks,
        )?;
        blk.fill(0);
        blk[0..4].copy_from_slice(&child_ino.to_le_bytes());
        blk[4..6].copy_from_slice(&4096u16.to_le_bytes());
        blk[6] = nbytes.len() as u8;
        blk[7] = file_type;
        blk[8..8 + nbytes.len()].copy_from_slice(nbytes);
        crate::ext4::superblock::write_block_4k(new_phys, &blk)?;

        parent_inode.i_size_lo = parent_inode.i_size_lo.saturating_add(4096);
        crate::ext4::inode::write_inode_disk(parent_ino, &parent_inode)
    }

    /// Remove named entry `name` from on-disk directory `parent_ino` and return removed inode.
    pub fn disk_remove_dir_entry(parent_ino: u32, name: &str) -> Result<u32, &'static str> {
        let parent_inode = crate::ext4::inode::read_inode_disk(parent_ino)?;
        if !parent_inode.is_dir() {
            return Err("EXT4: Parent is not a directory");
        }
        let num_blocks = parent_inode.file_size().div_ceil(4096).max(1) as u32;
        let mut blk = [0u8; 4096];

        for lblk in 0..num_blocks {
            let phys = match crate::ext4::inode::resolve_logical_block_disk(&parent_inode, lblk)? {
                Some(b) => b,
                None => continue,
            };
            crate::ext4::superblock::read_block_4k(phys, &mut blk)?;

            let mut off = 0usize;
            let mut prev_off: Option<usize> = None;
            while off + 8 <= 4096 {
                let inode =
                    u32::from_le_bytes([blk[off], blk[off + 1], blk[off + 2], blk[off + 3]]);
                let rec_len = u16::from_le_bytes([blk[off + 4], blk[off + 5]]) as usize;
                let name_len = blk[off + 6] as usize;
                if rec_len < 8 || off + rec_len > 4096 {
                    break;
                }

                if inode != 0 && off + 8 + name_len <= 4096 {
                    if let Ok(entry_name) = core::str::from_utf8(&blk[off + 8..off + 8 + name_len])
                    {
                        if entry_name == name {
                            if let Some(p_off) = prev_off {
                                let prev_rec =
                                    u16::from_le_bytes([blk[p_off + 4], blk[p_off + 5]]) as usize;
                                let merged = (prev_rec + rec_len) as u16;
                                blk[p_off + 4..p_off + 6].copy_from_slice(&merged.to_le_bytes());
                            } else {
                                blk[off..off + 4].copy_from_slice(&0u32.to_le_bytes());
                            }
                            crate::ext4::superblock::write_block_4k(phys, &blk)?;
                            return Ok(inode);
                        }
                    }
                }

                prev_off = Some(off);
                off += rec_len;
            }
        }

        Err("EXT4: Directory entry not found")
    }

    /// Compact `(inode, d_type, name_buf, name_len)` tuple returned by EXT4 directory scans.
    pub type Ext4DirTuple = (u32, u8, [u8; 64], usize);

    /// Retrieve the `target_idx`-th valid directory entry for POSIX `getdents`.
    pub fn disk_get_dir_entry(
        dir_ino: u32,
        target_idx: usize,
    ) -> Result<Option<Ext4DirTuple>, &'static str> {
        let dir_inode = crate::ext4::inode::read_inode_disk(dir_ino)?;
        if !dir_inode.is_dir() {
            return Err("EXT4: Target is not a directory");
        }
        let num_blocks = dir_inode.file_size().div_ceil(4096).max(1) as u32;
        let mut blk = [0u8; 4096];
        let mut cur = 0usize;

        for lblk in 0..num_blocks {
            let phys = match crate::ext4::inode::resolve_logical_block_disk(&dir_inode, lblk)? {
                Some(b) => b,
                None => continue,
            };
            crate::ext4::superblock::read_block_4k(phys, &mut blk)?;

            let mut off = 0usize;
            while off + 8 <= 4096 {
                let inode =
                    u32::from_le_bytes([blk[off], blk[off + 1], blk[off + 2], blk[off + 3]]);
                let rec_len = u16::from_le_bytes([blk[off + 4], blk[off + 5]]) as usize;
                let name_len = blk[off + 6] as usize;
                let file_type = blk[off + 7];
                if rec_len < 8 || off + rec_len > 4096 {
                    break;
                }
                if inode != 0 && off + 8 + name_len <= 4096 {
                    if cur == target_idx {
                        let mut nbuf = [0u8; 64];
                        let copy_len = name_len.min(64);
                        nbuf[..copy_len].copy_from_slice(&blk[off + 8..off + 8 + copy_len]);
                        let d_type = match file_type {
                            EXT4_FT_DIR => 4,
                            crate::ext4::dir::EXT4_FT_SYMLINK => 10,
                            _ => 8,
                        };
                        return Ok(Some((inode, d_type, nbuf, copy_len)));
                    }
                    cur += 1;
                }
                off += rec_len;
            }
        }
        Ok(None)
    }

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

    #[cfg(test)]
    static mut DATA_LBA_2048: [u8; 512] = [0u8; 512];
    #[cfg(test)]
    static mut DATA_LBA_2050: [u8; 512] = [0u8; 512];
    #[cfg(test)]
    static mut DATA_BLOCKS_INITIALIZED: bool = false;

    #[cfg(test)]
    fn ensure_data_blocks_initialized() {
        unsafe {
            if DATA_BLOCKS_INITIALIZED {
                return;
            }

            DATA_LBA_2048[..4].copy_from_slice(b"\x7FELF");
            DATA_LBA_2048[4] = 2;
            DATA_LBA_2048[5] = 1;

            let ver = b"Keira Kernel (EXT4 Driver Active)\n";
            DATA_LBA_2050[..ver.len()].copy_from_slice(ver);

            DATA_BLOCKS_INITIALIZED = true;
        }
    }

    pub fn read_file_content(path: &str, buf: &mut [u8]) -> Result<usize, &'static str> {
        let (_, inode) = lookup_path(path)?;
        if !inode.is_regular_file() {
            return Err("Target is not a regular file");
        }

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

        #[cfg(test)]
        {
            ensure_data_blocks_initialized();
            let sector_data = unsafe {
                match physical_lba {
                    2048 => DATA_LBA_2048,
                    2050 => DATA_LBA_2050,
                    _ => return Err("EXT4 unmapped data block sector"),
                }
            };

            let to_copy = file_size.min(512);
            buf[..to_copy].copy_from_slice(&sector_data[..to_copy]);
            Ok(to_copy)
        }
        #[cfg(not(test))]
        {
            let _ = (physical_lba, file_size);
            Err("No EXT4 block device attached")
        }
    }
}

pub use self::entry::*;
pub use self::lookup::*;
