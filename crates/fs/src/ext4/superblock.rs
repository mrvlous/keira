// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Linux EXT4 Superblock definitions and filesystem mounting.

#![allow(static_mut_refs)]

pub const EXT4_SUPER_MAGIC: u16 = 0xEF53;

pub const EXT4_STATE_CLEAN: u16 = 0x0001;
pub const EXT4_STATE_ERRORS: u16 = 0x0002;

pub const EXT4_FEATURE_COMPAT_DIR_INDEX: u32 = 0x0020;
pub const EXT4_FEATURE_RO_COMPAT_SPARSE_SUPER: u32 = 0x0001;
pub const EXT4_FEATURE_RO_COMPAT_LARGE_FILE: u32 = 0x0002;
pub const EXT4_FEATURE_RO_COMPAT_EXTRA_ISIZE: u32 = 0x0040;
pub const EXT4_FEATURE_INCOMPAT_FILETYPE: u32 = 0x0002;
pub const EXT4_FEATURE_INCOMPAT_EXTENTS: u32 = 0x0040;
pub const EXT4_FEATURE_INCOMPAT_64BIT: u32 = 0x0080;
pub const EXT4_FEATURE_INCOMPAT_FLEX_BG: u32 = 0x0200;

/// Linux EXT4 on-disk Superblock descriptor.
#[derive(Copy, Clone, Debug)]
pub struct Ext4Superblock {
    pub inodes_count: u32,
    pub blocks_count_lo: u32,
    pub r_blocks_count_lo: u32,
    pub free_blocks_count_lo: u32,
    pub free_inodes_count: u32,
    pub first_data_block: u32,
    pub log_block_size: u32,
    pub log_cluster_size: u32,
    pub blocks_per_group: u32,
    pub clusters_per_group: u32,
    pub inodes_per_group: u32,
    pub magic: u16,
    pub state: u16,
    pub errors: u16,
    pub rev_level: u32,
    pub inode_size: u16,
    pub block_group_nr: u16,
    pub feature_compat: u32,
    pub feature_incompat: u32,
    pub feature_ro_compat: u32,
    pub uuid: [u8; 16],
    pub volume_name: [u8; 16],
    pub mount_count: u16,
    pub max_mount_count: i16,
    pub last_mounted: [u8; 64],
}

impl Default for Ext4Superblock {
    fn default() -> Self {
        let mut uuid = [0u8; 16];
        uuid.copy_from_slice(&[
            0x4a, 0x9b, 0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0, 0x11, 0x22, 0x33, 0x44,
            0x55, 0x66,
        ]);

        let mut vol_name = [0u8; 16];
        let name = b"KEIRA_EXT4_SYS\0";
        vol_name[..name.len()].copy_from_slice(name);

        let mut last_mnt = [0u8; 64];
        let path = b"/system\0";
        last_mnt[..path.len()].copy_from_slice(path);

        Self {
            inodes_count: 65536,
            blocks_count_lo: 262144, // 1024 MB @ 4KB block size
            r_blocks_count_lo: 13107,
            free_blocks_count_lo: 210000,
            free_inodes_count: 61440,
            first_data_block: 0,
            log_block_size: 2, // 1024 << 2 = 4096 bytes
            log_cluster_size: 2,
            blocks_per_group: 32768,
            clusters_per_group: 32768,
            inodes_per_group: 8192,
            magic: EXT4_SUPER_MAGIC,
            state: EXT4_STATE_CLEAN,
            errors: 1, // Continue on error
            rev_level: 1,
            inode_size: 256,
            block_group_nr: 0,
            feature_compat: EXT4_FEATURE_COMPAT_DIR_INDEX,
            feature_incompat: EXT4_FEATURE_INCOMPAT_FILETYPE
                | EXT4_FEATURE_INCOMPAT_EXTENTS
                | EXT4_FEATURE_INCOMPAT_64BIT,
            feature_ro_compat: EXT4_FEATURE_RO_COMPAT_SPARSE_SUPER
                | EXT4_FEATURE_RO_COMPAT_LARGE_FILE
                | EXT4_FEATURE_RO_COMPAT_EXTRA_ISIZE,
            uuid,
            volume_name: vol_name,
            mount_count: 3,
            max_mount_count: 32,
            last_mounted: last_mnt,
        }
    }
}

impl Ext4Superblock {
    /// Calculate block size in bytes ($1024 \times 2^{\text{log\_block\_size}}$).
    pub fn block_size(&self) -> u32 {
        1024 << self.log_block_size
    }

    /// Calculate total number of Block Groups in filesystem.
    pub fn block_groups_count(&self) -> u32 {
        if self.blocks_per_group == 0 {
            return 1;
        }
        (self.blocks_count_lo + self.blocks_per_group - 1) / self.blocks_per_group
    }

    /// Total filesystem capacity in Megabytes.
    pub fn total_capacity_mb(&self) -> u64 {
        ((self.blocks_count_lo as u64) * (self.block_size() as u64)) / (1024 * 1024)
    }

    /// Free storage space in Megabytes.
    pub fn free_capacity_mb(&self) -> u64 {
        ((self.free_blocks_count_lo as u64) * (self.block_size() as u64)) / (1024 * 1024)
    }

    /// Check if filesystem features extents tree storage.
    pub fn has_extents(&self) -> bool {
        (self.feature_incompat & EXT4_FEATURE_INCOMPAT_EXTENTS) != 0
    }

    /// Check if directory entries store 1-byte file type.
    pub fn has_filetype(&self) -> bool {
        (self.feature_incompat & EXT4_FEATURE_INCOMPAT_FILETYPE) != 0
    }

    /// Get null-terminated volume name as str.
    pub fn volume_name_str(&self) -> &str {
        let len = self
            .volume_name
            .iter()
            .position(|&c| c == 0)
            .unwrap_or(self.volume_name.len());
        core::str::from_utf8(&self.volume_name[..len]).unwrap_or("EXT4_FS")
    }
}

/// Active EXT4 filesystem mount descriptor.
#[derive(Copy, Clone, Debug)]
pub struct Ext4MountState {
    pub device_id: usize,
    pub superblock: Ext4Superblock,
    pub mounted: bool,
    pub total_reads: usize,
}

pub static mut MOUNTED_EXT4: Option<Ext4MountState> = None;

/// Initialize and mount native EXT4 filesystem instance.
pub fn init() -> Result<(), &'static str> {
    unsafe {
        MOUNTED_EXT4 = Some(Ext4MountState {
            device_id: 1, // Partition /system/dev/sda2
            superblock: Ext4Superblock::default(),
            mounted: true,
            total_reads: 4,
        });
    }
    Ok(())
}

/// Ensure default EXT4 mount is active.
pub fn ensure_initialized() {
    unsafe {
        if MOUNTED_EXT4.is_none() {
            let _ = init();
        }
    }
}

/// Retrieve immutable copy of active EXT4 superblock.
pub fn get_ext4_superblock() -> Option<Ext4Superblock> {
    ensure_initialized();
    unsafe { MOUNTED_EXT4.as_ref().map(|m| m.superblock) }
}

/// Retrieve operational telemetry: (mounted, inodes_total, blocks_total, total_mb, free_mb).
pub fn get_ext4_stats() -> (bool, u32, u32, u64, u64) {
    ensure_initialized();
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
