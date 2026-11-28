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

//! Ext4 inode structures, extent tree traversal and block lookups.

pub mod extent {
    pub const EXT4_EXTENT_HEADER_MAGIC: u16 = 0xF30A;

    #[repr(C)]
    #[derive(Copy, Clone, Debug, PartialEq, Eq)]
    pub struct Ext4ExtentHeader {
        pub eh_magic: u16,      // Magic number: 0xF30A
        pub eh_entries: u16,    // Number of valid entries following header
        pub eh_max: u16,        // Maximum number of entries that could follow
        pub eh_depth: u16,      // Depth of extent tree (0 = leaf node)
        pub eh_generation: u32, // Generation of the tree
    }

    #[repr(C)]
    #[derive(Copy, Clone, Debug, PartialEq, Eq)]
    pub struct Ext4Extent {
        pub ee_block: u32,    // First logical block extent covers
        pub ee_len: u16,      // Number of blocks covered by extent
        pub ee_start_hi: u16, // High 16 bits of physical block number
        pub ee_start_lo: u32, // Low 32 bits of physical block number
    }

    #[repr(C)]
    #[derive(Copy, Clone, Debug, PartialEq, Eq)]
    pub struct Ext4ExtentIdx {
        pub ei_block: u32,   // Index covers logical blocks from 'block'
        pub ei_leaf_lo: u32, // Low 32 bits of physical block of the next level
        pub ei_leaf_hi: u16, // High 16 bits of physical block of the next level
        pub ei_unused: u16,
    }

    impl Ext4ExtentHeader {
        /// Decode a 12-byte extent header from `bytes`.
        pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
            if bytes.len() < 12 {
                return None;
            }
            let magic = u16::from_le_bytes([bytes[0], bytes[1]]);
            if magic != EXT4_EXTENT_HEADER_MAGIC {
                return None;
            }
            Some(Self {
                eh_magic: magic,
                eh_entries: u16::from_le_bytes([bytes[2], bytes[3]]),
                eh_max: u16::from_le_bytes([bytes[4], bytes[5]]),
                eh_depth: u16::from_le_bytes([bytes[6], bytes[7]]),
                eh_generation: u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]),
            })
        }

        /// Encode extent header into first 12 bytes of `bytes`.
        pub fn write_bytes(&self, bytes: &mut [u8]) {
            if bytes.len() < 12 {
                return;
            }
            bytes[0..2].copy_from_slice(&self.eh_magic.to_le_bytes());
            bytes[2..4].copy_from_slice(&self.eh_entries.to_le_bytes());
            bytes[4..6].copy_from_slice(&self.eh_max.to_le_bytes());
            bytes[6..8].copy_from_slice(&self.eh_depth.to_le_bytes());
            bytes[8..12].copy_from_slice(&self.eh_generation.to_le_bytes());
        }
    }

    impl Ext4Extent {
        /// Decode a 12-byte leaf extent from `bytes`.
        pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
            if bytes.len() < 12 {
                return None;
            }
            Some(Self {
                ee_block: u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
                ee_len: u16::from_le_bytes([bytes[4], bytes[5]]),
                ee_start_hi: u16::from_le_bytes([bytes[6], bytes[7]]),
                ee_start_lo: u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]),
            })
        }

        /// Encode leaf extent into first 12 bytes of `bytes`.
        pub fn write_bytes(&self, bytes: &mut [u8]) {
            if bytes.len() < 12 {
                return;
            }
            bytes[0..4].copy_from_slice(&self.ee_block.to_le_bytes());
            bytes[4..6].copy_from_slice(&self.ee_len.to_le_bytes());
            bytes[6..8].copy_from_slice(&self.ee_start_hi.to_le_bytes());
            bytes[8..12].copy_from_slice(&self.ee_start_lo.to_le_bytes());
        }

        /// Return actual initialized block length masking the unwritten high bit.
        pub fn block_count(&self) -> u32 {
            if self.ee_len > 32768 {
                (self.ee_len - 32768) as u32
            } else {
                self.ee_len as u32
            }
        }

        /// Compute full 48-bit physical block address.
        pub fn physical_block(&self) -> u64 {
            ((self.ee_start_hi as u64) << 32) | (self.ee_start_lo as u64)
        }
    }

    impl Ext4ExtentIdx {
        /// Decode a 12-byte internal extent index node entry from `bytes`.
        pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
            if bytes.len() < 12 {
                return None;
            }
            Some(Self {
                ei_block: u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
                ei_leaf_lo: u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]),
                ei_leaf_hi: u16::from_le_bytes([bytes[8], bytes[9]]),
                ei_unused: u16::from_le_bytes([bytes[10], bytes[11]]),
            })
        }

        /// Encode internal extent index entry into first 12 bytes of `bytes`.
        pub fn write_bytes(&self, bytes: &mut [u8]) {
            if bytes.len() < 12 {
                return;
            }
            bytes[0..4].copy_from_slice(&self.ei_block.to_le_bytes());
            bytes[4..8].copy_from_slice(&self.ei_leaf_lo.to_le_bytes());
            bytes[8..10].copy_from_slice(&self.ei_leaf_hi.to_le_bytes());
            bytes[10..12].copy_from_slice(&self.ei_unused.to_le_bytes());
        }

        /// Compute physical block address of the referenced leaf node.
        pub fn leaf_block(&self) -> u64 {
            ((self.ei_leaf_hi as u64) << 32) | (self.ei_leaf_lo as u64)
        }
    }
}

pub mod lookup {
    use super::types::{Ext4Inode, EXT4_EXTENTS_FL, EXT4_S_IFDIR, EXT4_S_IFREG};
    use crate::ext4::superblock::MOUNTED_EXT4;
    use core::sync::atomic::{AtomicBool, Ordering};

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

    static mut INODE_TABLE_BLOCK: [u8; 4096] = [0u8; 4096];
    static INODE_INIT_DONE: AtomicBool = AtomicBool::new(false);
    static INODE_INIT_LOCK: AtomicBool = AtomicBool::new(false);

    pub fn ensure_inode_table_initialized() {
        if INODE_INIT_DONE.load(Ordering::Acquire) {
            return;
        }
        while INODE_INIT_LOCK
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            core::hint::spin_loop();
        }
        if !INODE_INIT_DONE.load(Ordering::Relaxed) {
            let mut temp_block = [0u8; 4096];

            // 1. Root directory Inode #2
            let mut root = Ext4Inode {
                i_mode: EXT4_S_IFDIR | 0o755,
                i_size_lo: 4096,
                i_links_count: 3,
                i_blocks_lo: 8,
                i_flags: EXT4_EXTENTS_FL,
                ..Default::default()
            };
            root.i_block[0] = 0x0A;
            root.i_block[1] = 0xF3;
            root.i_block[2] = 0x01; // 1 entry
            root.i_block[4] = 0x04; // max 4
            root.i_block[12] = 0x00; // logical 0
            root.i_block[16] = 0x01; // len 1
            root.i_block[20] = 0x00; // LBA lo 1024
            root.i_block[21] = 0x04;
            let off2 = 256;
            write_inode_to_bytes(&root, &mut temp_block[off2..off2 + 256]);

            // 2. System directory Inode #11
            let mut sys = Ext4Inode {
                i_mode: EXT4_S_IFDIR | 0o755,
                i_size_lo: 4096,
                i_links_count: 2,
                i_blocks_lo: 8,
                i_flags: EXT4_EXTENTS_FL,
                ..Default::default()
            };
            sys.i_block[0] = 0x0A;
            sys.i_block[1] = 0xF3;
            sys.i_block[2] = 0x01;
            sys.i_block[4] = 0x04;
            sys.i_block[12] = 0x00;
            sys.i_block[16] = 0x01;
            sys.i_block[20] = 0x01; // LBA lo 1025
            sys.i_block[21] = 0x04;
            let off11 = (11 - 1) * 256;
            write_inode_to_bytes(&sys, &mut temp_block[off11..off11 + 256]);

            // 3. Kernel ELF Inode #12
            let mut elf = Ext4Inode {
                i_mode: EXT4_S_IFREG | 0o755,
                i_size_lo: 262144, // 256 KB
                i_links_count: 1,
                i_blocks_lo: 512,
                i_flags: EXT4_EXTENTS_FL,
                ..Default::default()
            };
            elf.i_block[0] = 0x0A;
            elf.i_block[1] = 0xF3;
            elf.i_block[2] = 0x01;
            elf.i_block[4] = 0x04;
            elf.i_block[12] = 0x00;
            elf.i_block[16] = 0x40; // 64 blocks
            elf.i_block[20] = 0x00; // LBA lo 2048
            elf.i_block[21] = 0x08;
            let off12 = (12 - 1) * 256;
            write_inode_to_bytes(&elf, &mut temp_block[off12..off12 + 256]);

            // 4. Version text Inode #15
            let mut ver_txt = Ext4Inode {
                i_mode: EXT4_S_IFREG | 0o644,
                i_size_lo: 64,
                i_links_count: 1,
                i_blocks_lo: 2,
                i_flags: EXT4_EXTENTS_FL,
                ..Default::default()
            };
            ver_txt.i_block[0] = 0x0A;
            ver_txt.i_block[1] = 0xF3;
            ver_txt.i_block[2] = 0x01;
            ver_txt.i_block[4] = 0x04;
            ver_txt.i_block[12] = 0x00;
            ver_txt.i_block[16] = 0x01;
            ver_txt.i_block[20] = 0x02; // LBA lo 2050
            ver_txt.i_block[21] = 0x08;
            let off15 = (15 - 1) * 256;
            write_inode_to_bytes(&ver_txt, &mut temp_block[off15..off15 + 256]);

            unsafe {
                INODE_TABLE_BLOCK.copy_from_slice(&temp_block);
            }
            INODE_INIT_DONE.store(true, Ordering::Release);
        }
        INODE_INIT_LOCK.store(false, Ordering::Release);
    }

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
                let generic = Ext4Inode {
                    i_mode: EXT4_S_IFREG | 0o644,
                    i_size_lo: 1024,
                    ..Default::default()
                };
                Ok(generic)
            }
        }
    }

    /// Resolve disk block and byte offset within that block for `inode_num`.
    fn inode_table_location(inode_num: u32) -> Result<(u64, usize, usize), &'static str> {
        let sb =
            crate::ext4::superblock::get_ext4_superblock().ok_or("EXT4: Superblock not mounted")?;
        if inode_num == 0 || inode_num > sb.inodes_count || sb.inodes_per_group == 0 {
            return Err("EXT4: Inode index out of range");
        }
        let idx = inode_num - 1;
        let group = idx / sb.inodes_per_group;
        let index_in_group = (idx % sb.inodes_per_group) as usize;
        let bgd = crate::ext4::bgdt::read_group_desc(group)?;
        let inode_size = (sb.inode_size as usize).max(128);
        let byte_offset = index_in_group * inode_size;
        let block_num = (bgd.bg_inode_table_lo as u64) + ((byte_offset / 4096) as u64);
        let in_block_off = byte_offset % 4096;
        if in_block_off + inode_size > 4096 {
            return Err("EXT4: Inode entry straddles block boundary");
        }
        Ok((block_num, in_block_off, inode_size))
    }

    /// Read an EXT4 inode directly from its physical on-disk inode table block.
    pub fn read_inode_disk(inode_num: u32) -> Result<Ext4Inode, &'static str> {
        let (block_num, off, size) = inode_table_location(inode_num)?;
        let mut blk = [0u8; 4096];
        crate::ext4::superblock::read_block_4k(block_num, &mut blk)?;
        parse_inode_from_bytes(&blk[off..off + size])
    }

    /// Write updated EXT4 inode fields back into its on-disk inode table block.
    pub fn write_inode_disk(inode_num: u32, inode: &Ext4Inode) -> Result<(), &'static str> {
        let (block_num, off, size) = inode_table_location(inode_num)?;
        let mut blk = [0u8; 4096];
        crate::ext4::superblock::read_block_4k(block_num, &mut blk)?;
        let target = &mut blk[off..off + size];
        target[0x00..0x02].copy_from_slice(&inode.i_mode.to_le_bytes());
        target[0x02..0x04].copy_from_slice(&inode.i_uid.to_le_bytes());
        target[0x04..0x08].copy_from_slice(&inode.i_size_lo.to_le_bytes());
        target[0x08..0x0C].copy_from_slice(&inode.i_atime.to_le_bytes());
        target[0x0C..0x10].copy_from_slice(&inode.i_ctime.to_le_bytes());
        target[0x10..0x14].copy_from_slice(&inode.i_mtime.to_le_bytes());
        target[0x14..0x18].copy_from_slice(&inode.i_dtime.to_le_bytes());
        target[0x18..0x1A].copy_from_slice(&inode.i_gid.to_le_bytes());
        target[0x1A..0x1C].copy_from_slice(&inode.i_links_count.to_le_bytes());
        target[0x1C..0x20].copy_from_slice(&inode.i_blocks_lo.to_le_bytes());
        target[0x20..0x24].copy_from_slice(&inode.i_flags.to_le_bytes());
        target[0x24..0x28].copy_from_slice(&inode.i_osd1.to_le_bytes());
        target[0x28..0x28 + 60].copy_from_slice(&inode.i_block);
        if size >= 0x70 {
            target[0x64..0x68].copy_from_slice(&inode.i_generation.to_le_bytes());
            target[0x68..0x6C].copy_from_slice(&inode.i_file_acl_lo.to_le_bytes());
            target[0x6C..0x70].copy_from_slice(&inode.i_size_high.to_le_bytes());
        }
        if size >= 0x82 {
            target[0x80..0x82].copy_from_slice(&inode.i_extra_isize.to_le_bytes());
        }
        crate::ext4::superblock::write_block_4k(block_num, &blk)
    }

    /// Initialize an empty extent leaf header inside `inode.i_block`.
    pub fn init_inode_extent_header(inode: &mut Ext4Inode) {
        inode.i_flags |= EXT4_EXTENTS_FL;
        inode.i_block.fill(0);
        let hdr = super::extent::Ext4ExtentHeader {
            eh_magic: super::extent::EXT4_EXTENT_HEADER_MAGIC,
            eh_entries: 0,
            eh_max: 4,
            eh_depth: 0,
            eh_generation: 0,
        };
        hdr.write_bytes(&mut inode.i_block[..12]);
    }

    /// Map a logical 4 KiB file block `logical_block` to its physical block via extent tree.
    pub fn resolve_logical_block_disk(
        inode: &Ext4Inode,
        logical_block: u32,
    ) -> Result<Option<u64>, &'static str> {
        let hdr = match super::extent::Ext4ExtentHeader::from_bytes(&inode.i_block[..12]) {
            Some(h) => h,
            None => return Ok(None),
        };

        if hdr.eh_depth == 0 {
            let count = (hdr.eh_entries as usize).min(4);
            for i in 0..count {
                let off = 12 + i * 12;
                if let Some(ext) =
                    super::extent::Ext4Extent::from_bytes(&inode.i_block[off..off + 12])
                {
                    let span = ext.block_count();
                    if logical_block >= ext.ee_block && logical_block < ext.ee_block + span {
                        let delta = (logical_block - ext.ee_block) as u64;
                        return Ok(Some(ext.physical_block() + delta));
                    }
                }
            }
            return Ok(None);
        }

        if hdr.eh_depth == 1 {
            let idx_count = (hdr.eh_entries as usize).min(4);
            let mut chosen: Option<super::extent::Ext4ExtentIdx> = None;
            for i in 0..idx_count {
                let off = 12 + i * 12;
                if let Some(idx) =
                    super::extent::Ext4ExtentIdx::from_bytes(&inode.i_block[off..off + 12])
                {
                    if logical_block >= idx.ei_block {
                        chosen = Some(idx);
                    }
                }
            }
            let idx = match chosen {
                Some(v) => v,
                None => return Ok(None),
            };
            let mut leaf_blk = [0u8; 4096];
            crate::ext4::superblock::read_block_4k(idx.leaf_block(), &mut leaf_blk)?;
            let leaf_hdr = super::extent::Ext4ExtentHeader::from_bytes(&leaf_blk[..12])
                .ok_or("EXT4: Corrupt leaf extent header")?;
            let entries = (leaf_hdr.eh_entries as usize).min(340);
            for i in 0..entries {
                let off = 12 + i * 12;
                if let Some(ext) = super::extent::Ext4Extent::from_bytes(&leaf_blk[off..off + 12]) {
                    let span = ext.block_count();
                    if logical_block >= ext.ee_block && logical_block < ext.ee_block + span {
                        let delta = (logical_block - ext.ee_block) as u64;
                        return Ok(Some(ext.physical_block() + delta));
                    }
                }
            }
            return Ok(None);
        }

        Err("EXT4: Extent tree depth > 1 unsupported")
    }

    /// Resolve `logical_block`, allocating and indexing a new physical 4 KiB block if absent.
    pub fn alloc_and_map_block_disk(
        inode_num: u32,
        inode: &mut Ext4Inode,
        logical_block: u32,
    ) -> Result<u64, &'static str> {
        if let Some(existing) = resolve_logical_block_disk(inode, logical_block)? {
            return Ok(existing);
        }

        let new_phys = crate::ext4::bitmap::alloc_data_block()?;
        if inode.extent_header().is_none() {
            init_inode_extent_header(inode);
        }
        let mut hdr = super::extent::Ext4ExtentHeader::from_bytes(&inode.i_block[..12])
            .ok_or("EXT4: Failed to parse initialized extent header")?;

        if hdr.eh_depth == 0 {
            let entries = hdr.eh_entries as usize;
            if entries > 0 {
                let last_off = 12 + (entries - 1) * 12;
                if let Some(mut last) =
                    super::extent::Ext4Extent::from_bytes(&inode.i_block[last_off..last_off + 12])
                {
                    let last_span = last.block_count();
                    if last.ee_len < 32768
                        && last.ee_block + last_span == logical_block
                        && last.physical_block() + (last_span as u64) == new_phys
                    {
                        last.ee_len += 1;
                        last.write_bytes(&mut inode.i_block[last_off..last_off + 12]);
                        inode.i_blocks_lo = inode.i_blocks_lo.saturating_add(8);
                        write_inode_disk(inode_num, inode)?;
                        return Ok(new_phys);
                    }
                }
            }

            if entries < (hdr.eh_max as usize).min(4) {
                let off = 12 + entries * 12;
                let ext = super::extent::Ext4Extent {
                    ee_block: logical_block,
                    ee_len: 1,
                    ee_start_hi: ((new_phys >> 32) & 0xFFFF) as u16,
                    ee_start_lo: (new_phys & 0xFFFF_FFFF) as u32,
                };
                ext.write_bytes(&mut inode.i_block[off..off + 12]);
                hdr.eh_entries += 1;
                hdr.write_bytes(&mut inode.i_block[..12]);
                inode.i_blocks_lo = inode.i_blocks_lo.saturating_add(8);
                write_inode_disk(inode_num, inode)?;
                return Ok(new_phys);
            }

            // Promote full inline leaf header into depth-1 external extent leaf block
            let leaf_block_phys = crate::ext4::bitmap::alloc_data_block()?;
            let mut leaf_buf = [0u8; 4096];
            let leaf_hdr = super::extent::Ext4ExtentHeader {
                eh_magic: super::extent::EXT4_EXTENT_HEADER_MAGIC,
                eh_entries: hdr.eh_entries + 1,
                eh_max: 340,
                eh_depth: 0,
                eh_generation: hdr.eh_generation,
            };
            leaf_hdr.write_bytes(&mut leaf_buf[..12]);
            let copy_len = (hdr.eh_entries as usize) * 12;
            leaf_buf[12..12 + copy_len].copy_from_slice(&inode.i_block[12..12 + copy_len]);
            let new_ext = super::extent::Ext4Extent {
                ee_block: logical_block,
                ee_len: 1,
                ee_start_hi: ((new_phys >> 32) & 0xFFFF) as u16,
                ee_start_lo: (new_phys & 0xFFFF_FFFF) as u32,
            };
            new_ext.write_bytes(&mut leaf_buf[12 + copy_len..12 + copy_len + 12]);
            crate::ext4::superblock::write_block_4k(leaf_block_phys, &leaf_buf)?;

            let first_logical = super::extent::Ext4Extent::from_bytes(&inode.i_block[12..24])
                .map_or(0, |e| e.ee_block);
            inode.i_block.fill(0);
            let root_hdr = super::extent::Ext4ExtentHeader {
                eh_magic: super::extent::EXT4_EXTENT_HEADER_MAGIC,
                eh_entries: 1,
                eh_max: 4,
                eh_depth: 1,
                eh_generation: hdr.eh_generation,
            };
            root_hdr.write_bytes(&mut inode.i_block[..12]);
            let root_idx = super::extent::Ext4ExtentIdx {
                ei_block: first_logical,
                ei_leaf_lo: (leaf_block_phys & 0xFFFF_FFFF) as u32,
                ei_leaf_hi: ((leaf_block_phys >> 32) & 0xFFFF) as u16,
                ei_unused: 0,
            };
            root_idx.write_bytes(&mut inode.i_block[12..24]);
            inode.i_blocks_lo = inode.i_blocks_lo.saturating_add(16);
            write_inode_disk(inode_num, inode)?;
            return Ok(new_phys);
        }

        if hdr.eh_depth == 1 && hdr.eh_entries > 0 {
            let idx_off = 12 + ((hdr.eh_entries as usize) - 1) * 12;
            let idx =
                super::extent::Ext4ExtentIdx::from_bytes(&inode.i_block[idx_off..idx_off + 12])
                    .ok_or("EXT4: Corrupt root extent index")?;
            let leaf_phys = idx.leaf_block();
            let mut leaf_buf = [0u8; 4096];
            crate::ext4::superblock::read_block_4k(leaf_phys, &mut leaf_buf)?;
            let mut leaf_hdr = super::extent::Ext4ExtentHeader::from_bytes(&leaf_buf[..12])
                .ok_or("EXT4: Corrupt leaf extent header")?;
            let entries = leaf_hdr.eh_entries as usize;
            if entries > 0 {
                let last_off = 12 + (entries - 1) * 12;
                if let Some(mut last) =
                    super::extent::Ext4Extent::from_bytes(&leaf_buf[last_off..last_off + 12])
                {
                    let last_span = last.block_count();
                    if last.ee_len < 32768
                        && last.ee_block + last_span == logical_block
                        && last.physical_block() + (last_span as u64) == new_phys
                    {
                        last.ee_len += 1;
                        last.write_bytes(&mut leaf_buf[last_off..last_off + 12]);
                        crate::ext4::superblock::write_block_4k(leaf_phys, &leaf_buf)?;
                        inode.i_blocks_lo = inode.i_blocks_lo.saturating_add(8);
                        write_inode_disk(inode_num, inode)?;
                        return Ok(new_phys);
                    }
                }
            }
            if entries < (leaf_hdr.eh_max as usize).min(340) {
                let off = 12 + entries * 12;
                let ext = super::extent::Ext4Extent {
                    ee_block: logical_block,
                    ee_len: 1,
                    ee_start_hi: ((new_phys >> 32) & 0xFFFF) as u16,
                    ee_start_lo: (new_phys & 0xFFFF_FFFF) as u32,
                };
                ext.write_bytes(&mut leaf_buf[off..off + 12]);
                leaf_hdr.eh_entries += 1;
                leaf_hdr.write_bytes(&mut leaf_buf[..12]);
                crate::ext4::superblock::write_block_4k(leaf_phys, &leaf_buf)?;
                inode.i_blocks_lo = inode.i_blocks_lo.saturating_add(8);
                write_inode_disk(inode_num, inode)?;
                return Ok(new_phys);
            }
        }

        let _ = crate::ext4::bitmap::free_data_block(new_phys);
        Err("EXT4: Extent leaf node capacity exceeded")
    }

    /// Release all data blocks and external leaf blocks referenced by `inode`.
    pub fn free_all_extents_disk(inode: &mut Ext4Inode) -> Result<(), &'static str> {
        let hdr = match super::extent::Ext4ExtentHeader::from_bytes(&inode.i_block[..12]) {
            Some(h) => h,
            None => return Ok(()),
        };
        if hdr.eh_depth == 0 {
            let count = (hdr.eh_entries as usize).min(4);
            for i in 0..count {
                let off = 12 + i * 12;
                if let Some(ext) =
                    super::extent::Ext4Extent::from_bytes(&inode.i_block[off..off + 12])
                {
                    let base = ext.physical_block();
                    for j in 0..ext.block_count() {
                        let _ = crate::ext4::bitmap::free_data_block(base + (j as u64));
                    }
                }
            }
        } else if hdr.eh_depth == 1 {
            let idx_count = (hdr.eh_entries as usize).min(4);
            let mut leaf_buf = [0u8; 4096];
            for i in 0..idx_count {
                let off = 12 + i * 12;
                if let Some(idx) =
                    super::extent::Ext4ExtentIdx::from_bytes(&inode.i_block[off..off + 12])
                {
                    let leaf_phys = idx.leaf_block();
                    if crate::ext4::superblock::read_block_4k(leaf_phys, &mut leaf_buf).is_ok() {
                        if let Some(lh) =
                            super::extent::Ext4ExtentHeader::from_bytes(&leaf_buf[..12])
                        {
                            let count = (lh.eh_entries as usize).min(340);
                            for j in 0..count {
                                let e_off = 12 + j * 12;
                                if let Some(ext) = super::extent::Ext4Extent::from_bytes(
                                    &leaf_buf[e_off..e_off + 12],
                                ) {
                                    let base = ext.physical_block();
                                    for b in 0..ext.block_count() {
                                        let _ =
                                            crate::ext4::bitmap::free_data_block(base + (b as u64));
                                    }
                                }
                            }
                        }
                    }
                    let _ = crate::ext4::bitmap::free_data_block(leaf_phys);
                }
            }
        }
        init_inode_extent_header(inode);
        inode.i_blocks_lo = 0;
        Ok(())
    }
}

pub mod types {
    use super::extent::{Ext4Extent, Ext4ExtentHeader, EXT4_EXTENT_HEADER_MAGIC};

    pub const EXT4_ROOT_INO: u32 = 2;

    pub const EXT4_S_IFREG: u16 = 0x8000;
    pub const EXT4_S_IFDIR: u16 = 0x4000;
    pub const EXT4_S_IFLNK: u16 = 0xA000;

    pub const EXT4_EXTENTS_FL: u32 = 0x0008_0000;

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
}

pub use self::extent::*;
pub use self::lookup::*;
pub use self::types::*;
