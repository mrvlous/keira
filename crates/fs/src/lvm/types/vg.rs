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
