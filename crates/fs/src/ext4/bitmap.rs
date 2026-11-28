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

//! Ext4 physical block bitmap and inode bitmap allocators.

use crate::ext4::bgdt::{read_group_desc, write_group_desc};
use crate::ext4::superblock::{read_block_4k, write_block_4k, MOUNTED_EXT4};

/// Allocate a single free 4 KiB physical block across EXT4 block groups and zero it.
pub fn alloc_data_block() -> Result<u64, &'static str> {
    let (groups_count, blocks_per_group, total_blocks, first_data) = unsafe {
        let m = MOUNTED_EXT4.as_ref().ok_or("EXT4: Volume not mounted")?;
        (
            m.superblock.block_groups_count(),
            m.superblock.blocks_per_group,
            m.superblock.blocks_count_lo,
            m.superblock.first_data_block as u64,
        )
    };

    let mut bmp = [0u8; 4096];
    for group in 0..groups_count {
        let mut bgd = read_group_desc(group)?;
        if bgd.bg_free_blocks_count_lo == 0 {
            continue;
        }
        read_block_4k(bgd.bg_block_bitmap_lo as u64, &mut bmp)?;

        let group_start = (group as u64) * (blocks_per_group as u64) + first_data;
        let max_bits = core::cmp::min(
            blocks_per_group as usize,
            (total_blocks as u64).saturating_sub(group_start) as usize,
        )
        .min(4096 * 8);

        for bit in 0..max_bits {
            let byte_idx = bit / 8;
            let bit_mask = 1u8 << (bit % 8);
            if (bmp[byte_idx] & bit_mask) == 0 {
                bmp[byte_idx] |= bit_mask;
                write_block_4k(bgd.bg_block_bitmap_lo as u64, &bmp)?;

                bgd.bg_free_blocks_count_lo = bgd.bg_free_blocks_count_lo.saturating_sub(1);
                write_group_desc(group, &bgd)?;

                unsafe {
                    if let Some(ref mut m) = MOUNTED_EXT4 {
                        m.superblock.free_blocks_count_lo =
                            m.superblock.free_blocks_count_lo.saturating_sub(1);
                        m.sb_dirty = true;
                    }
                }

                let phys_block = group_start + (bit as u64);
                let zeros = [0u8; 4096];
                write_block_4k(phys_block, &zeros)?;
                return Ok(phys_block);
            }
        }
    }

    Err("EXT4: No space left on device")
}

/// Release a previously allocated 4 KiB physical block back to its block bitmap.
pub fn free_data_block(phys_block: u64) -> Result<(), &'static str> {
    let (blocks_per_group, first_data, total_blocks) = unsafe {
        let m = MOUNTED_EXT4.as_ref().ok_or("EXT4: Volume not mounted")?;
        (
            m.superblock.blocks_per_group as u64,
            m.superblock.first_data_block as u64,
            m.superblock.blocks_count_lo as u64,
        )
    };

    if phys_block < first_data || phys_block >= total_blocks || blocks_per_group == 0 {
        return Err("EXT4: Block index out of bounds");
    }

    let rel = phys_block - first_data;
    let group = (rel / blocks_per_group) as u32;
    let bit = (rel % blocks_per_group) as usize;
    if bit >= 4096 * 8 {
        return Err("EXT4: Bitmap bit index out of range");
    }

    let mut bgd = read_group_desc(group)?;
    let mut bmp = [0u8; 4096];
    read_block_4k(bgd.bg_block_bitmap_lo as u64, &mut bmp)?;

    let byte_idx = bit / 8;
    let bit_mask = 1u8 << (bit % 8);
    if (bmp[byte_idx] & bit_mask) != 0 {
        bmp[byte_idx] &= !bit_mask;
        write_block_4k(bgd.bg_block_bitmap_lo as u64, &bmp)?;
        bgd.bg_free_blocks_count_lo = bgd.bg_free_blocks_count_lo.saturating_add(1);
        write_group_desc(group, &bgd)?;
        unsafe {
            if let Some(ref mut m) = MOUNTED_EXT4 {
                m.superblock.free_blocks_count_lo =
                    m.superblock.free_blocks_count_lo.saturating_add(1);
                m.sb_dirty = true;
            }
        }
    }

    Ok(())
}

/// Allocate a free EXT4 inode number, reserving reserved system inodes in group 0.
pub fn alloc_inode(is_dir: bool) -> Result<u32, &'static str> {
    let (groups_count, inodes_per_group, first_ino) = unsafe {
        let m = MOUNTED_EXT4.as_ref().ok_or("EXT4: Volume not mounted")?;
        (
            m.superblock.block_groups_count(),
            m.superblock.inodes_per_group,
            m.superblock.first_ino.max(11),
        )
    };

    let mut bmp = [0u8; 4096];
    for group in 0..groups_count {
        let mut bgd = read_group_desc(group)?;
        if bgd.bg_free_inodes_count_lo == 0 {
            continue;
        }
        read_block_4k(bgd.bg_inode_bitmap_lo as u64, &mut bmp)?;

        let start_bit = if group == 0 {
            first_ino.saturating_sub(1) as usize
        } else {
            0
        };
        let max_bits = (inodes_per_group as usize).min(4096 * 8);

        for bit in start_bit..max_bits {
            let byte_idx = bit / 8;
            let bit_mask = 1u8 << (bit % 8);
            if (bmp[byte_idx] & bit_mask) == 0 {
                bmp[byte_idx] |= bit_mask;
                write_block_4k(bgd.bg_inode_bitmap_lo as u64, &bmp)?;

                bgd.bg_free_inodes_count_lo = bgd.bg_free_inodes_count_lo.saturating_sub(1);
                if is_dir {
                    bgd.bg_used_dirs_count_lo = bgd.bg_used_dirs_count_lo.saturating_add(1);
                }
                write_group_desc(group, &bgd)?;

                unsafe {
                    if let Some(ref mut m) = MOUNTED_EXT4 {
                        m.superblock.free_inodes_count =
                            m.superblock.free_inodes_count.saturating_sub(1);
                        m.sb_dirty = true;
                    }
                }

                let inode_num = group * inodes_per_group + (bit as u32) + 1;
                return Ok(inode_num);
            }
        }
    }

    Err("EXT4: Inode table capacity exhausted")
}

/// Release a previously allocated EXT4 inode back to its inode bitmap.
pub fn free_inode(inode_num: u32, is_dir: bool) -> Result<(), &'static str> {
    let (inodes_per_group, total_inodes) = unsafe {
        let m = MOUNTED_EXT4.as_ref().ok_or("EXT4: Volume not mounted")?;
        (m.superblock.inodes_per_group, m.superblock.inodes_count)
    };
    if inode_num == 0 || inode_num > total_inodes || inodes_per_group == 0 {
        return Err("EXT4: Invalid inode number for release");
    }

    let idx = inode_num - 1;
    let group = idx / inodes_per_group;
    let bit = (idx % inodes_per_group) as usize;
    if bit >= 4096 * 8 {
        return Err("EXT4: Inode bitmap bit index out of range");
    }

    let mut bgd = read_group_desc(group)?;
    let mut bmp = [0u8; 4096];
    read_block_4k(bgd.bg_inode_bitmap_lo as u64, &mut bmp)?;

    let byte_idx = bit / 8;
    let bit_mask = 1u8 << (bit % 8);
    if (bmp[byte_idx] & bit_mask) != 0 {
        bmp[byte_idx] &= !bit_mask;
        write_block_4k(bgd.bg_inode_bitmap_lo as u64, &bmp)?;
        bgd.bg_free_inodes_count_lo = bgd.bg_free_inodes_count_lo.saturating_add(1);
        if is_dir {
            bgd.bg_used_dirs_count_lo = bgd.bg_used_dirs_count_lo.saturating_sub(1);
        }
        write_group_desc(group, &bgd)?;
        unsafe {
            if let Some(ref mut m) = MOUNTED_EXT4 {
                m.superblock.free_inodes_count = m.superblock.free_inodes_count.saturating_add(1);
                m.sb_dirty = true;
            }
        }
    }

    Ok(())
}
