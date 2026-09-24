// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Cluster scanning and allocation engine for FAT16.

use crate::fat::table::{read_sector, write_sector};
use crate::fat::types::Fat16Volume;
use crate::fat::volume::cluster_to_sector;

/// Scans the File Allocation Tables, reserves an unallocated cluster (marking it 0xFFFF), and zeros disk contents.
///
/// # Safety
///
/// Reads and writes raw sectors across all mirror copies of the FAT table on disk.
pub unsafe fn alloc_cluster(vol: &Fat16Volume) -> Result<u16, &'static str> {
    let mut sector_data = [0u8; 512];

    for s in 0..vol.sectors_per_fat {
        let sector = vol.fat_start_sector + s as u32;
        read_sector(sector, &mut sector_data)?;

        let entries = sector_data.as_mut_ptr() as *mut u16;
        for i in 0..256 {
            let cluster_idx = (s * 256) + i as u16;
            if cluster_idx < 2 {
                continue;
            }

            if *entries.add(i) == 0 {
                *entries.add(i) = 0xFFFF;

                for f in 0..vol.num_fats {
                    let fat_sec =
                        vol.fat_start_sector + (f as u32 * vol.sectors_per_fat as u32) + s as u32;
                    write_sector(fat_sec, &sector_data)?;
                }

                let first_sector = cluster_to_sector(cluster_idx, vol);
                let zero_buf = [0u8; 512];
                for cs in 0..vol.sectors_per_cluster as u32 {
                    write_sector(first_sector + cs, &zero_buf)?;
                }

                return Ok(cluster_idx);
            }
        }
    }

    Err("Disk is full (no free clusters)")
}
