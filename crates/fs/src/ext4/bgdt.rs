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

//! Ext4 Block Group Descriptor Table (BGDT) parser and persistence helpers.

use crate::ext4::superblock::{get_ext4_superblock, read_block_4k, write_block_4k};

/// On-disk Block Group Descriptor metadata fields.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct Ext4GroupDesc {
    pub bg_block_bitmap_lo: u32,
    pub bg_inode_bitmap_lo: u32,
    pub bg_inode_table_lo: u32,
    pub bg_free_blocks_count_lo: u16,
    pub bg_free_inodes_count_lo: u16,
    pub bg_used_dirs_count_lo: u16,
}

impl Ext4GroupDesc {
    /// Parse a single Block Group Descriptor entry from a 32-byte slice.
    pub fn parse_from_bytes(bytes: &[u8]) -> Result<Self, &'static str> {
        if bytes.len() < 32 {
            return Err("EXT4: Group descriptor slice smaller than 32 bytes");
        }
        Ok(Self {
            bg_block_bitmap_lo: u32::from_le_bytes([
                bytes[0x00],
                bytes[0x01],
                bytes[0x02],
                bytes[0x03],
            ]),
            bg_inode_bitmap_lo: u32::from_le_bytes([
                bytes[0x04],
                bytes[0x05],
                bytes[0x06],
                bytes[0x07],
            ]),
            bg_inode_table_lo: u32::from_le_bytes([
                bytes[0x08],
                bytes[0x09],
                bytes[0x0A],
                bytes[0x0B],
            ]),
            bg_free_blocks_count_lo: u16::from_le_bytes([bytes[0x0C], bytes[0x0D]]),
            bg_free_inodes_count_lo: u16::from_le_bytes([bytes[0x0E], bytes[0x0F]]),
            bg_used_dirs_count_lo: u16::from_le_bytes([bytes[0x10], bytes[0x11]]),
        })
    }

    /// Write mutated allocation fields back into an existing descriptor byte slice.
    pub fn patch_bytes(&self, bytes: &mut [u8]) {
        if bytes.len() < 32 {
            return;
        }
        bytes[0x00..0x04].copy_from_slice(&self.bg_block_bitmap_lo.to_le_bytes());
        bytes[0x04..0x08].copy_from_slice(&self.bg_inode_bitmap_lo.to_le_bytes());
        bytes[0x08..0x0C].copy_from_slice(&self.bg_inode_table_lo.to_le_bytes());
        bytes[0x0C..0x0E].copy_from_slice(&self.bg_free_blocks_count_lo.to_le_bytes());
        bytes[0x0E..0x10].copy_from_slice(&self.bg_free_inodes_count_lo.to_le_bytes());
        bytes[0x10..0x12].copy_from_slice(&self.bg_used_dirs_count_lo.to_le_bytes());
    }
}

/// Locate the BGDT filesystem block index and byte offset for `group`.
fn bgdt_location(group: u32) -> Result<(u64, usize, usize), &'static str> {
    let sb = get_ext4_superblock().ok_or("EXT4: Superblock not mounted")?;
    let desc_size = sb.group_desc_size().max(32);
    let bgdt_start_block = (sb.first_data_block as u64) + 1;
    let total_byte_off = (group as usize) * desc_size;
    let block_offset = total_byte_off / 4096;
    let in_block_off = total_byte_off % 4096;
    if in_block_off + desc_size > 4096 {
        return Err("EXT4: Group descriptor straddles block boundary");
    }
    Ok((
        bgdt_start_block + block_offset as u64,
        in_block_off,
        desc_size,
    ))
}

/// Read Block Group Descriptor `group` from the mounted EXT4 volume.
pub fn read_group_desc(group: u32) -> Result<Ext4GroupDesc, &'static str> {
    let (block_num, off, desc_size) = bgdt_location(group)?;
    let mut block_buf = [0u8; 4096];
    read_block_4k(block_num, &mut block_buf)?;
    Ext4GroupDesc::parse_from_bytes(&block_buf[off..off + desc_size])
}

/// Persist updated Block Group Descriptor `group` back to disk.
pub fn write_group_desc(group: u32, desc: &Ext4GroupDesc) -> Result<(), &'static str> {
    let (block_num, off, desc_size) = bgdt_location(group)?;
    let mut block_buf = [0u8; 4096];
    read_block_4k(block_num, &mut block_buf)?;
    desc.patch_bytes(&mut block_buf[off..off + desc_size]);
    write_block_4k(block_num, &block_buf)
}
