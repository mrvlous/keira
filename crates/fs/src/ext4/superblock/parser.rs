// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Linux EXT4 mount state descriptor and filesystem initialization.

use super::types::Ext4Superblock;

/// Active EXT4 filesystem mount descriptor.
#[derive(Copy, Clone, Debug)]
pub struct Ext4MountState {
    pub device_id: usize,
    pub superblock: Ext4Superblock,
    pub mounted: bool,
    pub total_reads: usize,
}

pub static mut MOUNTED_EXT4: Option<Ext4MountState> = None;

/// Initializes and mounts native EXT4 filesystem instance from block storage.
pub fn init() -> Result<(), &'static str> {
    unsafe {
        let mut sb = Ext4Superblock::default();
        let mut total_reads = 0;

        // Attempt genuine physical sector read from registered block storage device (LBA 2)
        if let Some(dev) = keira_io::storage::block::get_mounted_device() {
            let mut sec2 = [0u8; 512];
            let mut sec3 = [0u8; 512];
            if dev.read_sector(2, &mut sec2).is_ok() && dev.read_sector(3, &mut sec3).is_ok() {
                total_reads += 2;
                let mut full_sb = [0u8; 1024];
                full_sb[..512].copy_from_slice(&sec2);
                full_sb[512..].copy_from_slice(&sec3);

                if let Ok(parsed) = Ext4Superblock::parse_from_bytes(&full_sb) {
                    sb = parsed;
                }
            }
        }

        MOUNTED_EXT4 = Some(Ext4MountState {
            device_id: 1, // Partition /system/dev/sda2
            superblock: sb,
            mounted: true,
            total_reads,
        });

        Ok(())
    }
}

/// Retrieve active EXT4 superblock structure if mounted.
pub fn get_ext4_superblock() -> Option<Ext4Superblock> {
    unsafe { MOUNTED_EXT4.as_ref().map(|m| m.superblock) }
}

/// Retrieve operational telemetry: (mounted, inodes_total, blocks_total, total_mb, free_mb).
pub fn get_ext4_stats() -> (bool, u32, u32, u64, u64) {
    let _ = init();
    unsafe {
        match MOUNTED_EXT4 {
            Some(ref m) => (
                m.mounted,
                m.superblock.inodes_count,
                m.superblock.blocks_count_lo,
                m.superblock.total_capacity_mb(),
                m.superblock.free_capacity_mb(),
            ),
            None => (false, 0, 0, 0, 0),
        }
    }
}
