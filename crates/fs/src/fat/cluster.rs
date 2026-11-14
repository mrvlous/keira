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

//! FAT cluster allocation, deallocation and cluster chain traversal.

pub mod allocator {
    use crate::fat::table::{read_sector, write_sector};
    use crate::fat::types::Fat16Volume;
    use crate::fat::volume::cluster_to_sector;

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
                        let fat_sec = vol.fat_start_sector
                            + (f as u32 * vol.sectors_per_fat as u32)
                            + s as u32;
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
}

pub mod deallocator {
    use crate::fat::table::{read_sector, write_sector};
    use crate::fat::types::Fat16Volume;

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

            let next_cluster =
                (sector_data[offset] as u16) | ((sector_data[offset + 1] as u16) << 8);

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
}

pub mod traversal {
    use crate::fat::table::read_sector;
    use crate::fat::types::Fat16Volume;

    pub unsafe fn fat_next_cluster(cluster: u16, vol: &Fat16Volume) -> Result<u16, &'static str> {
        let fat_offset = cluster as u32 * 2;
        let sector = vol.fat_start_sector + (fat_offset / 512);
        let offset = (fat_offset % 512) as usize;

        let mut sector_data = [0u8; 512];
        read_sector(sector, &mut sector_data)?;

        let next = (sector_data[offset] as u16) | ((sector_data[offset + 1] as u16) << 8);
        Ok(next)
    }
}

pub use self::allocator::*;
pub use self::deallocator::*;
pub use self::traversal::*;
