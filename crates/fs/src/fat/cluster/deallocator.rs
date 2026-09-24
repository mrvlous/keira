// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Cluster chain deallocation and FAT table zeroing.

use crate::fat::table::{read_sector, write_sector};
use crate::fat::types::Fat16Volume;

/// Frees an entire linked cluster chain starting at `start_cluster` by clearing FAT entries to zero.
///
/// # Safety
///
/// Modifies block device sectors across all mirror copies of the FAT table on disk.
pub unsafe fn free_cluster_chain(
    start_cluster: u16,
    vol: &Fat16Volume,
) -> Result<(), &'static str> {
    let mut current_cluster = start_cluster;

    while (2..0xFFF8).contains(&current_cluster) {
        let fat_offset = current_cluster as u32 * 2;
        let s = fat_offset / 512;
        let offset = (fat_offset % 512) as usize;

        let sector = vol.fat_start_sector + s;
        let mut sector_data = [0u8; 512];
        read_sector(sector, &mut sector_data)?;

        let next_cluster = (sector_data[offset] as u16) | ((sector_data[offset + 1] as u16) << 8);

        sector_data[offset] = 0;
        sector_data[offset + 1] = 0;

        for f in 0..vol.num_fats {
            let fat_sec = vol.fat_start_sector + (f as u32 * vol.sectors_per_fat as u32) + s;
            write_sector(fat_sec, &sector_data)?;
        }

        current_cluster = next_cluster;
    }

    Ok(())
}
