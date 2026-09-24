// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Linux EXT4 Inode on-disk structure and file permission constants.

use super::extent::{Ext4Extent, Ext4ExtentHeader, EXT4_EXTENT_HEADER_MAGIC};

pub const EXT4_ROOT_INO: u32 = 2;

pub const EXT4_S_IFREG: u16 = 0x8000;
pub const EXT4_S_IFDIR: u16 = 0x4000;
pub const EXT4_S_IFLNK: u16 = 0xA000;

pub const EXT4_EXTENTS_FL: u32 = 0x0008_0000;

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
    pub i_osd1: u32,        // System dependent 1
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
    /// Returns true if inode represents a directory.
    pub fn is_dir(&self) -> bool {
        (self.i_mode & 0xF000) == EXT4_S_IFDIR
    }

    /// Returns true if inode represents a regular file.
    pub fn is_regular_file(&self) -> bool {
        (self.i_mode & 0xF000) == EXT4_S_IFREG
    }

    /// Returns true if inode represents a symbolic link.
    pub fn is_symlink(&self) -> bool {
        (self.i_mode & 0xF000) == EXT4_S_IFLNK
    }

    /// Full 64-bit file size in bytes combining lower and upper size registers.
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

    /// Parses inline extent header from `i_block`.
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

    /// Parses first leaf extent if depth == 0.
    pub fn first_extent(&self) -> Option<Ext4Extent> {
        let header = self.extent_header()?;
        if header.eh_depth != 0 || header.eh_entries == 0 {
            return None;
        }
        let offset = 12;
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
