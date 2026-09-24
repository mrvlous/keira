// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! FAT cluster chain traversal and linked entry lookup.

use crate::fat::table::read_sector;
use crate::fat::types::Fat16Volume;

/// Reads the FAT table entry for `cluster` to determine the subsequent cluster in the chain.
///
/// # Safety
///
/// Reads block device sectors via the FAT table cache.
pub unsafe fn fat_next_cluster(cluster: u16, vol: &Fat16Volume) -> Result<u16, &'static str> {
    let fat_offset = cluster as u32 * 2;
    let sector = vol.fat_start_sector + (fat_offset / 512);
    let offset = (fat_offset % 512) as usize;

    let mut sector_data = [0u8; 512];
    read_sector(sector, &mut sector_data)?;

    let next = (sector_data[offset] as u16) | ((sector_data[offset + 1] as u16) << 8);
    Ok(next)
}
