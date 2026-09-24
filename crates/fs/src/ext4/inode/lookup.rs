// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Inode table lookup, validation, serialization, and synthetic table initialization.

use super::types::{Ext4Inode, EXT4_EXTENTS_FL, EXT4_S_IFDIR, EXT4_S_IFREG};
use crate::ext4::superblock::MOUNTED_EXT4;
use core::sync::atomic::{AtomicBool, Ordering};

/// Validates whether an EXT4 inode number falls within legal bounds.
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

/// Parses a 256-byte on-disk EXT4 inode structure from raw bytes.
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

/// Serializes an EXT4 inode structure into on-disk binary format.
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

/// Ensures on-disk binary inode table is formatted and populated.
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
        let mut root = Ext4Inode::default();
        root.i_mode = EXT4_S_IFDIR | 0o755;
        root.i_size_lo = 4096;
        root.i_links_count = 3;
        root.i_blocks_lo = 8;
        root.i_flags = EXT4_EXTENTS_FL;
        root.i_block[0] = 0x0A;
        root.i_block[1] = 0xF3;
        root.i_block[2] = 0x01; // 1 entry
        root.i_block[4] = 0x04; // max 4
        root.i_block[12] = 0x00; // logical 0
        root.i_block[16] = 0x01; // len 1
        root.i_block[20] = 0x00; // LBA lo 1024
        root.i_block[21] = 0x04;
        let off2 = (2 - 1) * 256;
        write_inode_to_bytes(&root, &mut temp_block[off2..off2 + 256]);

        // 2. System directory Inode #11
        let mut sys = Ext4Inode::default();
        sys.i_mode = EXT4_S_IFDIR | 0o755;
        sys.i_size_lo = 4096;
        sys.i_links_count = 2;
        sys.i_blocks_lo = 8;
        sys.i_flags = EXT4_EXTENTS_FL;
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
        let mut elf = Ext4Inode::default();
        elf.i_mode = EXT4_S_IFREG | 0o755;
        elf.i_size_lo = 262144; // 256 KB
        elf.i_links_count = 1;
        elf.i_blocks_lo = 512;
        elf.i_flags = EXT4_EXTENTS_FL;
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

        // 4. Boot config Inode #14
        let mut boot_cfg = Ext4Inode::default();
        boot_cfg.i_mode = EXT4_S_IFREG | 0o644;
        boot_cfg.i_size_lo = 128;
        boot_cfg.i_links_count = 1;
        boot_cfg.i_blocks_lo = 2;
        boot_cfg.i_flags = EXT4_EXTENTS_FL;
        boot_cfg.i_block[0] = 0x0A;
        boot_cfg.i_block[1] = 0xF3;
        boot_cfg.i_block[2] = 0x01;
        boot_cfg.i_block[4] = 0x04;
        boot_cfg.i_block[12] = 0x00;
        boot_cfg.i_block[16] = 0x01;
        boot_cfg.i_block[20] = 0x01; // LBA lo 2049
        boot_cfg.i_block[21] = 0x08;
        let off14 = (14 - 1) * 256;
        write_inode_to_bytes(&boot_cfg, &mut temp_block[off14..off14 + 256]);

        // 5. Version text Inode #15
        let mut ver_txt = Ext4Inode::default();
        ver_txt.i_mode = EXT4_S_IFREG | 0o644;
        ver_txt.i_size_lo = 64;
        ver_txt.i_links_count = 1;
        ver_txt.i_blocks_lo = 2;
        ver_txt.i_flags = EXT4_EXTENTS_FL;
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

/// Reads inode attributes from the EXT4 on-disk inode table.
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
