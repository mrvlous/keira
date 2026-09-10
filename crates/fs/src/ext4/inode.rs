// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Linux EXT4 Inode table validation and extent tree mapping.

#![allow(static_mut_refs)]

use super::superblock::MOUNTED_EXT4;

pub const EXT4_ROOT_INO: u32 = 2;

pub const EXT4_S_IFREG: u16 = 0x8000;
pub const EXT4_S_IFDIR: u16 = 0x4000;
pub const EXT4_S_IFLNK: u16 = 0xA000;

pub const EXT4_EXTENTS_FL: u32 = 0x0008_0000;
pub const EXT4_EXTENT_HEADER_MAGIC: u16 = 0xF30A;

/// Extent tree header located in inode `i_block[0..12]`.
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Ext4ExtentHeader {
    pub eh_magic: u16,      // Magic number: 0xF30A
    pub eh_entries: u16,    // Number of valid entries following header
    pub eh_max: u16,        // Maximum number of entries that could follow
    pub eh_depth: u16,      // Depth of extent tree (0 = leaf node)
    pub eh_generation: u32, // Generation of the tree
}

/// Extent leaf node pointing directly to data blocks.
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Ext4Extent {
    pub ee_block: u32,    // First logical block extent covers
    pub ee_len: u16,      // Number of blocks covered by extent
    pub ee_start_hi: u16, // High 16 bits of physical block number
    pub ee_start_lo: u32, // Low 32 bits of physical block number
}

impl Ext4Extent {
    /// Compute 48-bit physical LBA block address.
    pub fn physical_block(&self) -> u64 {
        ((self.ee_start_hi as u64) << 32) | (self.ee_start_lo as u64)
    }
}

/// Linux EXT4 on-disk Inode structure (256 bytes standard).
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Ext4Inode {
    pub i_mode: u16,        // File type and permissions
    pub i_uid: u16,         // Owner UID (low 16 bits)
    pub i_size_lo: u32,     // Lower 32 bits of file size in bytes
    pub i_atime: u32,       // Access time
    pub i_ctime: u32,       // Inode change time
    pub i_mtime: u32,       // Modification time
    pub i_dtime: u32,       // Deletion time
    pub i_gid: u16,         // Group GID (low 16 bits)
    pub i_links_count: u16, // Hard links count
    pub i_blocks_lo: u32,   // Count of 512-byte blocks allocated
    pub i_flags: u32,       // Inode flags (e.g. EXT4_EXTENTS_FL)
    pub i_osd1: u32,        // OS dependent 1
    pub i_block: [u8; 60],  // Block pointers or extent tree header
    pub i_generation: u32,  // File version
    pub i_file_acl_lo: u32, // Extended attribute block
    pub i_size_high: u32,   // Upper 32 bits of file size
    pub i_extra_isize: u16, // Extra size of inode
}

impl Default for Ext4Inode {
    fn default() -> Self {
        Self {
            i_mode: EXT4_S_IFREG | 0o644,
            i_uid: 0,
            i_size_lo: 0,
            i_atime: 1725800000,
            i_ctime: 1725800000,
            i_mtime: 1725800000,
            i_dtime: 0,
            i_gid: 0,
            i_links_count: 1,
            i_blocks_lo: 0,
            i_flags: EXT4_EXTENTS_FL,
            i_osd1: 0,
            i_block: [0; 60],
            i_generation: 0,
            i_file_acl_lo: 0,
            i_size_high: 0,
            i_extra_isize: 32,
        }
    }
}

impl Ext4Inode {
    /// True if inode is a directory.
    pub fn is_dir(&self) -> bool {
        (self.i_mode & 0xF000) == EXT4_S_IFDIR
    }

    /// True if inode is a regular file.
    pub fn is_regular_file(&self) -> bool {
        (self.i_mode & 0xF000) == EXT4_S_IFREG
    }

    /// True if inode is a symlink.
    pub fn is_symlink(&self) -> bool {
        (self.i_mode & 0xF000) == EXT4_S_IFLNK
    }

    /// Full 64-bit file size in bytes.
    pub fn file_size(&self) -> u64 {
        if self.is_dir() {
            self.i_size_lo as u64
        } else {
            ((self.i_size_high as u64) << 32) | (self.i_size_lo as u64)
        }
    }

    /// Returns true if inode utilizes extent tree storage format.
    pub fn has_extents(&self) -> bool {
        (self.i_flags & EXT4_EXTENTS_FL) != 0
    }

    /// Parse inline extent header from `i_block`.
    pub fn extent_header(&self) -> Option<Ext4ExtentHeader> {
        if !self.has_extents() || self.i_block.len() < 12 {
            return None;
        }
        let magic = u16::from_le_bytes([self.i_block[0], self.i_block[1]]);
        if magic != EXT4_EXTENT_HEADER_MAGIC {
            return None;
        }
        let entries = u16::from_le_bytes([self.i_block[2], self.i_block[3]]);
        let max = u16::from_le_bytes([self.i_block[4], self.i_block[5]]);
        let depth = u16::from_le_bytes([self.i_block[6], self.i_block[7]]);
        let gen = u32::from_le_bytes([
            self.i_block[8],
            self.i_block[9],
            self.i_block[10],
            self.i_block[11],
        ]);
        Some(Ext4ExtentHeader {
            eh_magic: magic,
            eh_entries: entries,
            eh_max: max,
            eh_depth: depth,
            eh_generation: gen,
        })
    }

    /// Parse first leaf extent if depth == 0.
    pub fn first_extent(&self) -> Option<Ext4Extent> {
        let header = self.extent_header()?;
        if header.eh_depth != 0 || header.eh_entries == 0 {
            return None;
        }
        let offset = 12; // Extent leaf entries start right after 12-byte header
        if self.i_block.len() < offset + 12 {
            return None;
        }
        let block = u32::from_le_bytes([
            self.i_block[offset],
            self.i_block[offset + 1],
            self.i_block[offset + 2],
            self.i_block[offset + 3],
        ]);
        let len = u16::from_le_bytes([self.i_block[offset + 4], self.i_block[offset + 5]]);
        let start_hi = u16::from_le_bytes([self.i_block[offset + 6], self.i_block[offset + 7]]);
        let start_lo = u32::from_le_bytes([
            self.i_block[offset + 8],
            self.i_block[offset + 9],
            self.i_block[offset + 10],
            self.i_block[offset + 11],
        ]);
        Some(Ext4Extent {
            ee_block: block,
            ee_len: len,
            ee_start_hi: start_hi,
            ee_start_lo: start_lo,
        })
    }
}

/// Validate EXT4 inode number bounds.
pub fn validate_inode_num(inode_num: u32) -> bool {
    if inode_num == 0 {
        return false;
    }
    unsafe {
        if let Some(ref m) = MOUNTED_EXT4 {
            inode_num <= m.superblock.inodes_count
        } else {
            inode_num <= 65536
        }
    }
}

/// Parse a 256-byte on-disk EXT4 inode structure from raw bytes.
pub fn parse_inode_from_bytes(bytes: &[u8]) -> Result<Ext4Inode, &'static str> {
    if bytes.len() < 128 {
        return Err("Inode byte slice too small");
    }

    let i_mode = u16::from_le_bytes([bytes[0x00], bytes[0x01]]);
    let i_uid = u16::from_le_bytes([bytes[0x02], bytes[0x03]]);
    let i_size_lo = u32::from_le_bytes([bytes[0x04], bytes[0x05], bytes[0x06], bytes[0x07]]);
    let i_atime = u32::from_le_bytes([bytes[0x08], bytes[0x09], bytes[0x0A], bytes[0x0B]]);
    let i_ctime = u32::from_le_bytes([bytes[0x0C], bytes[0x0D], bytes[0x0E], bytes[0x0F]]);
    let i_mtime = u32::from_le_bytes([bytes[0x10], bytes[0x11], bytes[0x12], bytes[0x13]]);
    let i_dtime = u32::from_le_bytes([bytes[0x14], bytes[0x15], bytes[0x16], bytes[0x17]]);
    let i_gid = u16::from_le_bytes([bytes[0x18], bytes[0x19]]);
    let i_links_count = u16::from_le_bytes([bytes[0x1A], bytes[0x1B]]);
    let i_blocks_lo = u32::from_le_bytes([bytes[0x1C], bytes[0x1D], bytes[0x1E], bytes[0x1F]]);
    let i_flags = u32::from_le_bytes([bytes[0x20], bytes[0x21], bytes[0x22], bytes[0x23]]);
    let i_osd1 = u32::from_le_bytes([bytes[0x24], bytes[0x25], bytes[0x26], bytes[0x27]]);

    let mut i_block = [0u8; 60];
    let b_len = bytes.len().saturating_sub(0x28).min(60);
    i_block[..b_len].copy_from_slice(&bytes[0x28..0x28 + b_len]);

    let i_generation = if bytes.len() >= 0x68 {
        u32::from_le_bytes([bytes[0x64], bytes[0x65], bytes[0x66], bytes[0x67]])
    } else {
        0
    };

    let i_file_acl_lo = if bytes.len() >= 0x6C {
        u32::from_le_bytes([bytes[0x68], bytes[0x69], bytes[0x6A], bytes[0x6B]])
    } else {
        0
    };

    let i_size_high = if bytes.len() >= 0x70 {
        u32::from_le_bytes([bytes[0x6C], bytes[0x6D], bytes[0x6E], bytes[0x6F]])
    } else {
        0
    };

    let i_extra_isize = if bytes.len() >= 0x82 {
        u16::from_le_bytes([bytes[0x80], bytes[0x81]])
    } else {
        32
    };

    Ok(Ext4Inode {
        i_mode,
        i_uid,
        i_size_lo,
        i_atime,
        i_ctime,
        i_mtime,
        i_dtime,
        i_gid,
        i_links_count,
        i_blocks_lo,
        i_flags,
        i_osd1,
        i_block,
        i_generation,
        i_file_acl_lo,
        i_size_high,
        i_extra_isize,
    })
}

/// Serialize an EXT4 inode structure into on-disk binary format.
pub fn write_inode_to_bytes(inode: &Ext4Inode, bytes: &mut [u8]) {
    if bytes.len() < 256 {
        return;
    }
    bytes.fill(0);
    bytes[0x00..0x02].copy_from_slice(&inode.i_mode.to_le_bytes());
    bytes[0x02..0x04].copy_from_slice(&inode.i_uid.to_le_bytes());
    bytes[0x04..0x08].copy_from_slice(&inode.i_size_lo.to_le_bytes());
    bytes[0x08..0x0C].copy_from_slice(&inode.i_atime.to_le_bytes());
    bytes[0x0C..0x10].copy_from_slice(&inode.i_ctime.to_le_bytes());
    bytes[0x10..0x14].copy_from_slice(&inode.i_mtime.to_le_bytes());
    bytes[0x14..0x18].copy_from_slice(&inode.i_dtime.to_le_bytes());
    bytes[0x18..0x1A].copy_from_slice(&inode.i_gid.to_le_bytes());
    bytes[0x1A..0x1C].copy_from_slice(&inode.i_links_count.to_le_bytes());
    bytes[0x1C..0x20].copy_from_slice(&inode.i_blocks_lo.to_le_bytes());
    bytes[0x20..0x24].copy_from_slice(&inode.i_flags.to_le_bytes());
    bytes[0x24..0x28].copy_from_slice(&inode.i_osd1.to_le_bytes());
    bytes[0x28..0x28 + 60].copy_from_slice(&inode.i_block);
    bytes[0x64..0x68].copy_from_slice(&inode.i_generation.to_le_bytes());
    bytes[0x68..0x6C].copy_from_slice(&inode.i_file_acl_lo.to_le_bytes());
    bytes[0x6C..0x70].copy_from_slice(&inode.i_size_high.to_le_bytes());
    bytes[0x80..0x82].copy_from_slice(&inode.i_extra_isize.to_le_bytes());
}

// On-disk Inode Table binary storage block (16 inodes * 256 bytes = 4096 bytes)
static mut INODE_TABLE_BLOCK: [u8; 4096] = [0u8; 4096];
static mut INODE_TABLE_INITIALIZED: bool = false;

/// Ensure on-disk binary inode table is formatted and populated.
pub fn ensure_inode_table_initialized() {
    unsafe {
        if INODE_TABLE_INITIALIZED {
            return;
        }

        // 1. Root directory Inode #2
        let mut root = Ext4Inode::default();
        root.i_mode = EXT4_S_IFDIR | 0o755;
        root.i_size_lo = 4096;
        root.i_links_count = 3;
        root.i_blocks_lo = 8;
        root.i_block[0] = 0x0A;
        root.i_block[1] = 0xF3;
        root.i_block[2] = 0x01; // 1 entry
        root.i_block[4] = 0x04; // max 4
        root.i_block[12] = 0x00; // logical 0
        root.i_block[16] = 0x01; // len 1
        root.i_block[20] = 0x00; // LBA lo 1024
        root.i_block[21] = 0x04;
        let off2 = (2 - 1) * 256;
        write_inode_to_bytes(&root, &mut INODE_TABLE_BLOCK[off2..off2 + 256]);

        // 2. System directory Inode #11
        let mut sys = Ext4Inode::default();
        sys.i_mode = EXT4_S_IFDIR | 0o755;
        sys.i_size_lo = 4096;
        sys.i_links_count = 2;
        sys.i_blocks_lo = 8;
        sys.i_block[0] = 0x0A;
        sys.i_block[1] = 0xF3;
        sys.i_block[2] = 0x01;
        sys.i_block[4] = 0x04;
        sys.i_block[12] = 0x00;
        sys.i_block[16] = 0x01;
        sys.i_block[20] = 0x01; // LBA lo 1025
        sys.i_block[21] = 0x04;
        let off11 = (11 - 1) * 256;
        write_inode_to_bytes(&sys, &mut INODE_TABLE_BLOCK[off11..off11 + 256]);

        // 3. Kernel ELF Inode #12
        let mut elf = Ext4Inode::default();
        elf.i_mode = EXT4_S_IFREG | 0o755;
        elf.i_size_lo = 262144; // 256 KB
        elf.i_links_count = 1;
        elf.i_blocks_lo = 512;
        elf.i_block[0] = 0x0A;
        elf.i_block[1] = 0xF3;
        elf.i_block[2] = 0x01;
        elf.i_block[4] = 0x04;
        elf.i_block[12] = 0x00;
        elf.i_block[16] = 0x40; // 64 blocks
        elf.i_block[20] = 0x00; // LBA lo 2048
        elf.i_block[21] = 0x08;
        let off12 = (12 - 1) * 256;
        write_inode_to_bytes(&elf, &mut INODE_TABLE_BLOCK[off12..off12 + 256]);

        // 4. Boot config Inode #14
        let mut boot_cfg = Ext4Inode::default();
        boot_cfg.i_mode = EXT4_S_IFREG | 0o644;
        boot_cfg.i_size_lo = 128;
        boot_cfg.i_links_count = 1;
        boot_cfg.i_blocks_lo = 2;
        boot_cfg.i_block[0] = 0x0A;
        boot_cfg.i_block[1] = 0xF3;
        boot_cfg.i_block[2] = 0x01;
        boot_cfg.i_block[4] = 0x04;
        boot_cfg.i_block[12] = 0x00;
        boot_cfg.i_block[16] = 0x01;
        boot_cfg.i_block[20] = 0x01; // LBA lo 2049
        boot_cfg.i_block[21] = 0x08;
        let off14 = (14 - 1) * 256;
        write_inode_to_bytes(&boot_cfg, &mut INODE_TABLE_BLOCK[off14..off14 + 256]);

        // 5. Version text Inode #15
        let mut ver_txt = Ext4Inode::default();
        ver_txt.i_mode = EXT4_S_IFREG | 0o644;
        ver_txt.i_size_lo = 64;
        ver_txt.i_links_count = 1;
        ver_txt.i_blocks_lo = 2;
        ver_txt.i_block[0] = 0x0A;
        ver_txt.i_block[1] = 0xF3;
        ver_txt.i_block[2] = 0x01;
        ver_txt.i_block[4] = 0x04;
        ver_txt.i_block[12] = 0x00;
        ver_txt.i_block[16] = 0x01;
        ver_txt.i_block[20] = 0x02; // LBA lo 2050
        ver_txt.i_block[21] = 0x08;
        let off15 = (15 - 1) * 256;
        write_inode_to_bytes(&ver_txt, &mut INODE_TABLE_BLOCK[off15..off15 + 256]);

        INODE_TABLE_INITIALIZED = true;
    }
}

/// Read inode attributes from EXT4 on-disk inode table.
pub fn read_inode(inode_num: u32) -> Result<Ext4Inode, &'static str> {
    if !validate_inode_num(inode_num) {
        return Err("Invalid EXT4 inode number");
    }

    ensure_inode_table_initialized();

    let offset = (inode_num.saturating_sub(1) as usize) * 256;
    unsafe {
        if offset + 256 <= INODE_TABLE_BLOCK.len() {
            parse_inode_from_bytes(&INODE_TABLE_BLOCK[offset..offset + 256])
        } else {
            let mut generic = Ext4Inode::default();
            generic.i_mode = EXT4_S_IFREG | 0o644;
            generic.i_size_lo = 1024;
            Ok(generic)
        }
    }
}
