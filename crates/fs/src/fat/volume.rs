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

//! FAT volume boot sector parsing, disk geometry and statistics.

pub mod boot {
    use crate::fat::table::{clear_cache, read_sector};
    use crate::fat::types::Fat16Volume;

    pub static mut VOLUME: Option<Fat16Volume> = None;

    pub static mut CURRENT_DIR_CLUSTER: u16 = 0;

    pub unsafe fn init() -> Result<(), &'static str> {
        clear_cache();
        let mut boot_sector = [0u8; 512];
        read_sector(0, &mut boot_sector)?;

        if boot_sector[510] != 0x55 || boot_sector[511] != 0xAA {
            return Err("FAT16 Init: Invalid boot sector signature (missing 0xAA55)");
        }

        let bytes_per_sector = (boot_sector[11] as u16) | ((boot_sector[12] as u16) << 8);
        let sectors_per_cluster = boot_sector[13];
        let reserved_sector_count = (boot_sector[14] as u16) | ((boot_sector[15] as u16) << 8);
        let num_fats = boot_sector[16];
        let root_entry_count = (boot_sector[17] as u16) | ((boot_sector[18] as u16) << 8);
        let sectors_per_fat = (boot_sector[22] as u16) | ((boot_sector[23] as u16) << 8);

        if bytes_per_sector != 512 {
            return Err("FAT16 Init: Only 512 bytes per sector is supported");
        }

        if sectors_per_fat == 0 {
            return Err("FAT16 Init: Sectors per FAT is 0 (FAT32 is not supported)");
        }

        let fat_start_sector = reserved_sector_count as u32;
        let root_dir_start_sector = fat_start_sector + (num_fats as u32 * sectors_per_fat as u32);
        let root_dir_size_sectors = (root_entry_count as u32 * 32).div_ceil(512);
        let data_start_sector = root_dir_start_sector + root_dir_size_sectors;

        let vol = Fat16Volume {
            bytes_per_sector,
            sectors_per_cluster,
            reserved_sector_count,
            num_fats,
            root_entry_count,
            sectors_per_fat,
            fat_start_sector,
            root_dir_start_sector,
            root_dir_size_sectors,
            data_start_sector,
        };

        VOLUME = Some(vol);
        CURRENT_DIR_CLUSTER = 0;
        Ok(())
    }
}

pub mod geometry {
    use crate::fat::types::Fat16Volume;

    pub fn cluster_to_sector(cluster: u16, vol: &Fat16Volume) -> u32 {
        vol.data_start_sector + ((cluster as u32 - 2) * vol.sectors_per_cluster as u32)
    }
}

pub mod info {
    use super::boot::VOLUME;
    use keira_io::storage::block::get_mounted_device;
    use keira_io::vga;

    pub unsafe fn print_disk_info() {
        if let Some(dev) = get_mounted_device() {
            let sectors = dev.get_size_sectors();
            vga::set_color(vga::Color::White, vga::Color::Black);
            vga::print_str("Active Drive (");
            vga::print_str(dev.get_name());
            vga::print_str(") Size: ");
            vga::print_u64((sectors as u64 * 512) / (1024 * 1024));
            vga::print_str(" MB (");
            vga::print_u64(sectors as u64);
            vga::print_str(" sectors)\n");
        } else {
            vga::set_color(vga::Color::LightRed, vga::Color::Black);
            vga::print_str("No active block device mounted\n");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            return;
        }

        let vol_ptr = &raw const VOLUME;
        if let Some(vol) = unsafe { (*vol_ptr).as_ref() } {
            vga::set_color(vga::Color::White, vga::Color::Black);
            vga::print_str("Filesystem:     ");
            vga::print_str("FAT16\n");

            vga::print_str("Cluster Size:   ");
            vga::print_u64(vol.sectors_per_cluster as u64 * 512);
            vga::print_str(" bytes (");
            vga::print_u64(vol.sectors_per_cluster as u64);
            vga::print_str(" sectors)\n");

            vga::print_str("Reserved Secs:  ");
            vga::print_u64(vol.reserved_sector_count as u64);
            vga::print_str("\n");

            vga::print_str("Root Directory: ");
            vga::print_u64(vol.root_entry_count as u64);
            vga::print_str(" entries (start sector: ");
            vga::print_u64(vol.root_dir_start_sector as u64);
            vga::print_str(")\n");
        } else {
            vga::set_color(vga::Color::Yellow, vga::Color::Black);
            vga::print_str("Filesystem:     Not a valid FAT16 partition\n");
        }

        let (hits, misses, evictions, active) = crate::fat::table::get_sector_cache_stats();
        let total_access = hits + misses;
        let hit_rate = (hits * 100).checked_div(total_access).unwrap_or(0);
        vga::set_color(vga::Color::White, vga::Color::Black);
        vga::print_str("LRU Cache:      ");
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        vga::print_u64(active as u64);
        vga::print_str("/16 slots (Hits: ");
        vga::print_u64(hits);
        vga::print_str(", Misses: ");
        vga::print_u64(misses);
        vga::print_str(", Evictions: ");
        vga::print_u64(evictions);
        vga::print_str(", Hit Ratio: ");
        vga::print_u64(hit_rate);
        vga::print_str("%)\n");

        let (p_hits, p_misses, p_evictions, p_writebacks, p_active) =
            crate::cache::get_page_cache_stats();
        let p_total = p_hits + p_misses;
        let p_hit_rate = (p_hits * 100).checked_div(p_total).unwrap_or(0);
        vga::set_color(vga::Color::White, vga::Color::Black);
        vga::print_str("Page Cache:     ");
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        vga::print_u64(p_active as u64);
        vga::print_str("/32 pages (Hits: ");
        vga::print_u64(p_hits);
        vga::print_str(", Misses: ");
        vga::print_u64(p_misses);
        vga::print_str(", Evictions: ");
        vga::print_u64(p_evictions);
        vga::print_str(", Writebacks: ");
        vga::print_u64(p_writebacks);
        vga::print_str(", Hit Ratio: ");
        vga::print_u64(p_hit_rate);
        vga::print_str("%)\n");
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    }
}

pub use self::boot::*;
pub use self::geometry::*;
pub use self::info::*;
