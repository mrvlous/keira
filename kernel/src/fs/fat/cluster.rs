// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//!
//! File Allocation Table (FAT) cluster chain traversal, allocation, and deallocation.

use super::types::Fat16Volume;
use super::volume::cluster_to_sector;
use super::{read_sector, write_sector};

/// Read the FAT entry for a cluster to find the next cluster
pub unsafe fn fat_next_cluster(cluster: u16, vol: &Fat16Volume) -> Result<u16, &'static str> {
    let fat_offset = cluster as u32 * 2;
    let sector = vol.fat_start_sector + (fat_offset / 512);
    let offset = (fat_offset % 512) as usize;

    let mut sector_data = [0u8; 512];
    read_sector(sector, &mut sector_data)?;

    let next = (sector_data[offset] as u16) | ((sector_data[offset + 1] as u16) << 8);
    Ok(next)
}

/// Scan both FAT tables to find and allocate an empty cluster (marked 0xFFFF)
pub unsafe fn alloc_cluster(vol: &Fat16Volume) -> Result<u16, &'static str> {
    let mut sector_data = [0u8; 512];

    for s in 0..vol.sectors_per_fat {
        let sector = vol.fat_start_sector + s as u32;
        read_sector(sector, &mut sector_data)?;

        let entries = sector_data.as_mut_ptr() as *mut u16;
        for i in 0..256 {
            let cluster_idx = (s * 256) + i as u16;
            if cluster_idx < 2 {
                // Skip reserved clusters 0 and 1
                continue;
            }

            if *entries.add(i) == 0 {
                // Found free cluster!
                // End of chain marker
                *entries.add(i) = 0xFFFF;

                // Write back to all FAT copies
                for f in 0..vol.num_fats {
                    let fat_sec =
                        vol.fat_start_sector + (f as u32 * vol.sectors_per_fat as u32) + s as u32;
                    write_sector(fat_sec, &sector_data)?;
                }

                // Zero out the newly allocated cluster's sectors
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

/// Free a cluster chain starting at `start_cluster` by marking entries as 0x0000 in both FATs
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

        // Read next cluster in the chain
        let next_cluster = (sector_data[offset] as u16) | ((sector_data[offset + 1] as u16) << 8);

        // Free current cluster entry
        sector_data[offset] = 0;
        sector_data[offset + 1] = 0;

        // Write updated FAT sector to all FAT copies
        for f in 0..vol.num_fats {
            let fat_sec = vol.fat_start_sector + (f as u32 * vol.sectors_per_fat as u32) + s;
            write_sector(fat_sec, &sector_data)?;
        }

        current_cluster = next_cluster;
    }

    Ok(())
}
