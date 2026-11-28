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

//! Ext4 superblock definitions, physical block I/O and descriptor parser.

pub mod parser {
    use super::types::{Ext4Superblock, EXT4_SUPER_MAGIC};
    use keira_io::storage::block::{get_device, BlockDevice};

    #[derive(Copy, Clone, Debug)]
    pub struct Ext4MountState {
        pub device_id: usize,
        pub device_name: &'static str,
        pub disk_backed: bool,
        pub superblock: Ext4Superblock,
        pub raw_sb: [u8; 1024],
        pub mounted: bool,
        pub sb_dirty: bool,
        pub total_reads: usize,
        pub total_writes: usize,
    }

    pub static mut MOUNTED_EXT4: Option<Ext4MountState> = None;

    const EXT4_PROBE_CANDIDATES: [&str; 6] = ["ahci1", "ext4", "ram1", "ram0", "ide0", "ahci0"];

    fn probe_device_ext4(dev: &dyn BlockDevice) -> Option<(Ext4Superblock, [u8; 1024])> {
        let mut sec2 = [0u8; 512];
        let mut sec3 = [0u8; 512];
        if dev.read_sector(2, &mut sec2).is_err() || dev.read_sector(3, &mut sec3).is_err() {
            return None;
        }
        let mut full_sb = [0u8; 1024];
        full_sb[..512].copy_from_slice(&sec2);
        full_sb[512..].copy_from_slice(&sec3);
        let sb = Ext4Superblock::parse_from_bytes(&full_sb).ok()?;
        if sb.magic != EXT4_SUPER_MAGIC {
            return None;
        }
        Some((sb, full_sb))
    }

    /// Initialize EXT4 mount state against an explicit registered block device name.
    pub fn init_with_device(dev_name: &'static str) -> Result<(), &'static str> {
        let dev = get_device(dev_name).ok_or("EXT4: Block device not found")?;
        let (mut sb, raw_sb) =
            probe_device_ext4(dev).ok_or("EXT4: Invalid superblock magic on device")?;
        sb.mount_count = sb.mount_count.saturating_add(1);
        unsafe {
            MOUNTED_EXT4 = Some(Ext4MountState {
                device_id: 2,
                device_name: dev_name,
                disk_backed: true,
                superblock: sb,
                raw_sb,
                mounted: true,
                sb_dirty: false,
                total_reads: 2,
                total_writes: 0,
            });
        }
        Ok(())
    }

    /// Probe storage devices for an EXT4 volume or fall back to in-memory metadata state.
    pub fn init() -> Result<(), &'static str> {
        unsafe {
            for candidate in EXT4_PROBE_CANDIDATES {
                if let Some(dev) = get_device(candidate) {
                    if let Some((mut sb, raw_sb)) = probe_device_ext4(dev) {
                        sb.mount_count = sb.mount_count.saturating_add(1);
                        MOUNTED_EXT4 = Some(Ext4MountState {
                            device_id: 2,
                            device_name: candidate,
                            disk_backed: true,
                            superblock: sb,
                            raw_sb,
                            mounted: true,
                            sb_dirty: false,
                            total_reads: 2,
                            total_writes: 0,
                        });
                        return Ok(());
                    }
                }
            }

            let sb = Ext4Superblock::default();
            let mut raw_sb = [0u8; 1024];
            sb.write_to_bytes(&mut raw_sb);
            MOUNTED_EXT4 = Some(Ext4MountState {
                device_id: 2,
                device_name: "none",
                disk_backed: false,
                superblock: sb,
                raw_sb,
                mounted: true,
                sb_dirty: false,
                total_reads: 0,
                total_writes: 0,
            });
            Ok(())
        }
    }

    /// Return whether an active on-disk EXT4 block volume is mounted.
    pub fn is_disk_mounted() -> bool {
        unsafe {
            matches!(
                MOUNTED_EXT4,
                Some(ref m) if m.mounted && m.disk_backed
            )
        }
    }

    /// Retrieve active EXT4 backing block device reference if mounted.
    pub fn active_ext4_device() -> Option<&'static dyn BlockDevice> {
        unsafe {
            match MOUNTED_EXT4 {
                Some(ref m) if m.disk_backed => get_device(m.device_name),
                _ => None,
            }
        }
    }

    /// Read one physical 4 KiB filesystem block (`block_num`) from the mounted EXT4 device.
    pub fn read_block_4k(block_num: u64, out: &mut [u8; 4096]) -> Result<(), &'static str> {
        let dev = active_ext4_device().ok_or("EXT4: No physical block device mounted")?;
        let base_lba = u32::try_from(block_num.saturating_mul(8))
            .map_err(|_| "EXT4: Block LBA overflows 32-bit sector index")?;
        let mut sec = [0u8; 512];
        for i in 0..8u32 {
            dev.read_sector(base_lba + i, &mut sec)?;
            let off = (i as usize) * 512;
            out[off..off + 512].copy_from_slice(&sec);
        }
        unsafe {
            if let Some(ref mut m) = MOUNTED_EXT4 {
                m.total_reads = m.total_reads.saturating_add(8);
            }
        }
        Ok(())
    }

    /// Write one physical 4 KiB filesystem block (`block_num`) to the mounted EXT4 device.
    pub fn write_block_4k(block_num: u64, data: &[u8; 4096]) -> Result<(), &'static str> {
        let dev = active_ext4_device().ok_or("EXT4: No physical block device mounted")?;
        let base_lba = u32::try_from(block_num.saturating_mul(8))
            .map_err(|_| "EXT4: Block LBA overflows 32-bit sector index")?;
        let mut sec = [0u8; 512];
        for i in 0..8u32 {
            let off = (i as usize) * 512;
            sec.copy_from_slice(&data[off..off + 512]);
            dev.write_sector(base_lba + i, &sec)?;
        }
        unsafe {
            if let Some(ref mut m) = MOUNTED_EXT4 {
                m.total_writes = m.total_writes.saturating_add(8);
            }
        }
        Ok(())
    }

    /// Flush updated superblock counters back to physical LBA sectors 2 and 3.
    pub fn sync_superblock() -> Result<(), &'static str> {
        let dev = match active_ext4_device() {
            Some(d) => d,
            None => return Ok(()),
        };
        unsafe {
            let state = match MOUNTED_EXT4 {
                Some(ref mut m) if m.disk_backed => m,
                _ => return Ok(()),
            };
            if !state.sb_dirty {
                return dev.flush();
            }
            state.superblock.patch_raw_bytes(&mut state.raw_sb);
            let mut sec2 = [0u8; 512];
            let mut sec3 = [0u8; 512];
            sec2.copy_from_slice(&state.raw_sb[..512]);
            sec3.copy_from_slice(&state.raw_sb[512..]);
            dev.write_sector(2, &sec2)?;
            dev.write_sector(3, &sec3)?;
            state.total_writes = state.total_writes.saturating_add(2);
            state.sb_dirty = false;
            dev.flush()
        }
    }

    /// Return a copy of the active EXT4 superblock if mounted.
    pub fn get_ext4_superblock() -> Option<Ext4Superblock> {
        unsafe { MOUNTED_EXT4.as_ref().map(|m| m.superblock) }
    }

    /// Initialize if needed and retrieve `(mounted, inodes, blocks, total_mb, free_mb)`.
    pub fn get_ext4_stats() -> (bool, u32, u32, u64, u64) {
        unsafe {
            if MOUNTED_EXT4.is_none() {
                let _ = init();
            }
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
}

pub mod types {
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
        pub first_ino: u32,
        pub inode_size: u16,
        pub block_group_nr: u16,
        pub desc_size: u16,
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
            let name = b"KEIRA_EXT4_ROOT\0";
            vol_name[..name.len()].copy_from_slice(name);

            let mut last_mnt = [0u8; 64];
            let path = b"/\0";
            last_mnt[..path.len()].copy_from_slice(path);

            Self {
                inodes_count: 65536,
                blocks_count_lo: 262144,
                r_blocks_count_lo: 13107,
                free_blocks_count_lo: 210000,
                free_inodes_count: 61440,
                first_data_block: 0,
                log_block_size: 2,
                log_cluster_size: 2,
                blocks_per_group: 32768,
                clusters_per_group: 32768,
                inodes_per_group: 8192,
                magic: EXT4_SUPER_MAGIC,
                state: EXT4_STATE_CLEAN,
                errors: 1,
                rev_level: 1,
                first_ino: 11,
                inode_size: 256,
                block_group_nr: 0,
                desc_size: 32,
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
        /// Calculate block size in bytes.
        pub fn block_size(&self) -> u32 {
            1024 << self.log_block_size
        }

        /// Calculate total number of Block Groups in filesystem.
        pub fn block_groups_count(&self) -> u32 {
            if self.blocks_per_group == 0 {
                return 1;
            }
            self.blocks_count_lo.div_ceil(self.blocks_per_group)
        }

        /// Return effective Block Group Descriptor byte stride (`32` or `64`).
        pub fn group_desc_size(&self) -> usize {
            if (self.feature_incompat & EXT4_FEATURE_INCOMPAT_64BIT) != 0 && self.desc_size >= 64 {
                self.desc_size as usize
            } else {
                32
            }
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

        /// Parse on-disk superblock directly from binary bytes (offset 1024 / LBA 2).
        pub fn parse_from_bytes(bytes: &[u8]) -> Result<Self, &'static str> {
            if bytes.len() < 1024 {
                return Err("Superblock buffer too small");
            }
            let magic = u16::from_le_bytes([bytes[0x38], bytes[0x39]]);
            if magic != EXT4_SUPER_MAGIC {
                return Err("Invalid EXT4 superblock magic");
            }

            let inodes_count =
                u32::from_le_bytes([bytes[0x00], bytes[0x01], bytes[0x02], bytes[0x03]]);
            let blocks_count_lo =
                u32::from_le_bytes([bytes[0x04], bytes[0x05], bytes[0x06], bytes[0x07]]);
            let r_blocks_count_lo =
                u32::from_le_bytes([bytes[0x08], bytes[0x09], bytes[0x0A], bytes[0x0B]]);
            let free_blocks_count_lo =
                u32::from_le_bytes([bytes[0x0C], bytes[0x0D], bytes[0x0E], bytes[0x0F]]);
            let free_inodes_count =
                u32::from_le_bytes([bytes[0x10], bytes[0x11], bytes[0x12], bytes[0x13]]);
            let first_data_block =
                u32::from_le_bytes([bytes[0x14], bytes[0x15], bytes[0x16], bytes[0x17]]);
            let log_block_size =
                u32::from_le_bytes([bytes[0x18], bytes[0x19], bytes[0x1A], bytes[0x1B]]);
            let log_cluster_size =
                u32::from_le_bytes([bytes[0x1C], bytes[0x1D], bytes[0x1E], bytes[0x1F]]);
            let blocks_per_group =
                u32::from_le_bytes([bytes[0x20], bytes[0x21], bytes[0x22], bytes[0x23]]);
            let clusters_per_group =
                u32::from_le_bytes([bytes[0x24], bytes[0x25], bytes[0x26], bytes[0x27]]);
            let inodes_per_group =
                u32::from_le_bytes([bytes[0x28], bytes[0x29], bytes[0x2A], bytes[0x2B]]);
            let state = u16::from_le_bytes([bytes[0x3A], bytes[0x3B]]);
            let errors = u16::from_le_bytes([bytes[0x3C], bytes[0x3D]]);
            let rev_level =
                u32::from_le_bytes([bytes[0x4C], bytes[0x4D], bytes[0x4E], bytes[0x4F]]);
            let first_ino_raw =
                u32::from_le_bytes([bytes[0x54], bytes[0x55], bytes[0x56], bytes[0x57]]);
            let first_ino = if first_ino_raw == 0 {
                11
            } else {
                first_ino_raw
            };
            let inode_size_raw = u16::from_le_bytes([bytes[0x58], bytes[0x59]]);
            let inode_size = if inode_size_raw == 0 {
                128
            } else {
                inode_size_raw
            };
            let block_group_nr = u16::from_le_bytes([bytes[0x5A], bytes[0x5B]]);
            let feature_compat =
                u32::from_le_bytes([bytes[0x5C], bytes[0x5D], bytes[0x5E], bytes[0x5F]]);
            let feature_incompat =
                u32::from_le_bytes([bytes[0x60], bytes[0x61], bytes[0x62], bytes[0x63]]);
            let feature_ro_compat =
                u32::from_le_bytes([bytes[0x64], bytes[0x65], bytes[0x66], bytes[0x67]]);

            let mut uuid = [0u8; 16];
            uuid.copy_from_slice(&bytes[0x68..0x78]);

            let mut volume_name = [0u8; 16];
            volume_name.copy_from_slice(&bytes[0x78..0x88]);

            let mount_count = u16::from_le_bytes([bytes[0x34], bytes[0x35]]);
            let max_mount_count = i16::from_le_bytes([bytes[0x36], bytes[0x37]]);

            let mut last_mounted = [0u8; 64];
            let copy_len = bytes.len().saturating_sub(0x88).min(64);
            last_mounted[..copy_len].copy_from_slice(&bytes[0x88..0x88 + copy_len]);

            let desc_size_raw = u16::from_le_bytes([bytes[0xFE], bytes[0xFF]]);
            let desc_size = if desc_size_raw == 0 {
                32
            } else {
                desc_size_raw
            };

            Ok(Self {
                inodes_count,
                blocks_count_lo,
                r_blocks_count_lo,
                free_blocks_count_lo,
                free_inodes_count,
                first_data_block,
                log_block_size,
                log_cluster_size,
                blocks_per_group,
                clusters_per_group,
                inodes_per_group,
                magic,
                state,
                errors,
                rev_level,
                first_ino,
                inode_size,
                block_group_nr,
                desc_size,
                feature_compat,
                feature_incompat,
                feature_ro_compat,
                uuid,
                volume_name,
                mount_count,
                max_mount_count,
                last_mounted,
            })
        }

        /// Update mutable counters inside an existing 1024-byte raw superblock buffer.
        pub fn patch_raw_bytes(&self, buffer: &mut [u8; 1024]) {
            buffer[0x0C..0x10].copy_from_slice(&self.free_blocks_count_lo.to_le_bytes());
            buffer[0x10..0x14].copy_from_slice(&self.free_inodes_count.to_le_bytes());
            buffer[0x34..0x36].copy_from_slice(&self.mount_count.to_le_bytes());
            buffer[0x3A..0x3C].copy_from_slice(&self.state.to_le_bytes());
        }

        /// Serialize superblock structure into binary bytes for on-disk write.
        pub fn write_to_bytes(&self, buffer: &mut [u8; 1024]) {
            buffer.fill(0);
            buffer[0x00..0x04].copy_from_slice(&self.inodes_count.to_le_bytes());
            buffer[0x04..0x08].copy_from_slice(&self.blocks_count_lo.to_le_bytes());
            buffer[0x08..0x0C].copy_from_slice(&self.r_blocks_count_lo.to_le_bytes());
            buffer[0x0C..0x10].copy_from_slice(&self.free_blocks_count_lo.to_le_bytes());
            buffer[0x10..0x14].copy_from_slice(&self.free_inodes_count.to_le_bytes());
            buffer[0x14..0x18].copy_from_slice(&self.first_data_block.to_le_bytes());
            buffer[0x18..0x1C].copy_from_slice(&self.log_block_size.to_le_bytes());
            buffer[0x1C..0x20].copy_from_slice(&self.log_cluster_size.to_le_bytes());
            buffer[0x20..0x24].copy_from_slice(&self.blocks_per_group.to_le_bytes());
            buffer[0x24..0x28].copy_from_slice(&self.clusters_per_group.to_le_bytes());
            buffer[0x28..0x2C].copy_from_slice(&self.inodes_per_group.to_le_bytes());
            buffer[0x34..0x36].copy_from_slice(&self.mount_count.to_le_bytes());
            buffer[0x36..0x38].copy_from_slice(&self.max_mount_count.to_le_bytes());
            buffer[0x38..0x3A].copy_from_slice(&self.magic.to_le_bytes());
            buffer[0x3A..0x3C].copy_from_slice(&self.state.to_le_bytes());
            buffer[0x3C..0x3E].copy_from_slice(&self.errors.to_le_bytes());
            buffer[0x4C..0x50].copy_from_slice(&self.rev_level.to_le_bytes());
            buffer[0x54..0x58].copy_from_slice(&self.first_ino.to_le_bytes());
            buffer[0x58..0x5A].copy_from_slice(&self.inode_size.to_le_bytes());
            buffer[0x5A..0x5C].copy_from_slice(&self.block_group_nr.to_le_bytes());
            buffer[0x5C..0x60].copy_from_slice(&self.feature_compat.to_le_bytes());
            buffer[0x60..0x64].copy_from_slice(&self.feature_incompat.to_le_bytes());
            buffer[0x64..0x68].copy_from_slice(&self.feature_ro_compat.to_le_bytes());
            buffer[0x68..0x78].copy_from_slice(&self.uuid);
            buffer[0x78..0x88].copy_from_slice(&self.volume_name);
            buffer[0xFE..0x100].copy_from_slice(&self.desc_size.to_le_bytes());
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
}

pub use self::parser::*;
pub use self::types::*;
