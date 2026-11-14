// SPDX-License-Identifier: GPL-2.0-only
/*
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra <mrvlous@proton.me>
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 */

//! Logical Volume Manager (LVM) types and volume group descriptors.

pub mod cmd {
    pub const LVM_CMD_INFO: u32 = 1;
    pub const LVM_CMD_CREATE_VG: u32 = 2;
    pub const LVM_CMD_CREATE_LV: u32 = 3;
    pub const LVM_CMD_RAID_STATUS: u32 = 4;
    pub const LVM_CMD_RAID_SYNC: u32 = 5;
}

pub mod lv {
    #[derive(Copy, Clone, Debug)]
    pub struct LogicalVolume {
        /// Logical partition name (e.g. `lv_root`).
        pub name: [u8; 16],
        /// Allocated partition size in Megabytes.
        pub size_mb: u32,
        /// Filesystem type signature (e.g. `fat16`, `ext4`).
        pub fstype: [u8; 8],
        /// Activation status flag.
        pub active: bool,
    }
}

pub mod pv {
    #[derive(Copy, Clone, Debug)]
    pub struct PhysicalVolume {
        /// Canonical device node name (e.g. `/dev/sda`).
        pub name: [u8; 16],
        /// Total capacity in Megabytes.
        pub size_mb: u32,
        /// Unallocated extent space in Megabytes.
        pub free_mb: u32,
        /// Allocation status indicating whether drive is incorporated into a Volume Group.
        pub in_use: bool,
    }
}

pub mod raid {
    #[derive(Copy, Clone, Debug)]
    pub struct RaidArray {
        /// Multi-disk device name (e.g. `md0`).
        pub name: [u8; 16],
        /// RAID redundancy topology level (0 = RAID-0 Striping, 1 = RAID-1 Mirroring).
        pub level: u8,
        /// Number of physical drives participating in the array.
        pub disk_count: u8,
        /// Mirror synchronization status flag.
        pub synced: bool,
        /// Usable aggregate capacity in Megabytes.
        pub size_mb: u32,
        /// Active state flag.
        pub active: bool,
    }
}

pub mod vg {
    use super::lv::LogicalVolume;
    
    #[derive(Copy, Clone, Debug)]
    pub struct VolumeGroup {
        /// Volume Group identifier (e.g. `vg_keira0`).
        pub name: [u8; 16],
        /// Total aggregate capacity in Megabytes.
        pub total_mb: u32,
        /// Available unallocated extent capacity in Megabytes.
        pub free_mb: u32,
        /// Count of physical volumes attached.
        pub pv_count: u8,
        /// Count of active logical volumes created.
        pub lv_count: u8,
        /// Fixed-size logical volume partition descriptors array.
        pub lvs: [LogicalVolume; 4],
        /// Active state flag.
        pub active: bool,
    }
}

pub use self::cmd::*;
pub use self::lv::*;
pub use self::pv::*;
pub use self::raid::*;
pub use self::vg::*;
