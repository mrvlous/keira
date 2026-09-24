// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Linux EXT4 extent tree nodes and physical block address mapping.

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

/// Extent leaf node pointing directly to contiguous physical data blocks.
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Ext4Extent {
    pub ee_block: u32,    // First logical block extent covers
    pub ee_len: u16,      // Number of blocks covered by extent
    pub ee_start_hi: u16, // High 16 bits of physical block number
    pub ee_start_lo: u32, // Low 32 bits of physical block number
}

impl Ext4Extent {
    /// Computes full 48-bit physical LBA block address.
    pub fn physical_block(&self) -> u64 {
        ((self.ee_start_hi as u64) << 32) | (self.ee_start_lo as u64)
    }
}
