// SPDX-License-Identifier: GPL-2.0-only

//! Volume Group (VG) storage pool topology.

use super::lv::LogicalVolume;

/// Volume Group pooling multiple Physical Volumes into an extent allocator.
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
