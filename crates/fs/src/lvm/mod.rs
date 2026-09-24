// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Logical Volume Manager (LVM) and Software RAID topology abstractions.
//!
//! Subdivided into specialized hyper-modular sub-packages:
//! - `types/`: Physical Volume, Volume Group, Logical Volume, and RAID data structures.
//! - `volume/`: Extent management, storage pool allocation, mirror sync, and syscall dispatcher.

pub mod types;
pub mod volume;

#[cfg(test)]
mod tests;

pub use self::types::{
    LogicalVolume, PhysicalVolume, RaidArray, VolumeGroup, LVM_CMD_CREATE_LV, LVM_CMD_CREATE_VG,
    LVM_CMD_INFO, LVM_CMD_RAID_STATUS, LVM_CMD_RAID_SYNC,
};
pub use self::volume::{
    create_lv_named, create_vg_named, get_lvm_stats, get_raid_arrays, get_raid_stats,
    get_vol_groups, sync_raid_array, sys_raid_lvm,
};
