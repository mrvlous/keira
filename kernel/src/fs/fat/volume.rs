// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

use super::types::Fat16Volume;
use super::{CURRENT_DIR_CLUSTER, VOLUME};
use crate::io::vga;

pub unsafe fn cluster_to_sector(cluster: u16, vol: &Fat16Volume) -> u32 {
    vol.data_start_sector + ((cluster as u32 - 2) * vol.sectors_per_cluster as u32)
}

/// Initialize the FAT16 driver by reading the BPB from Sector 0
pub unsafe fn init() -> Result<(), &'static str> {
    super::clear_cache();
    let mut boot_sector = [0u8; 512];
    super::read_sector(0, &mut boot_sector)?;

    // Check signature 0xAA55 at the end of sector
    if boot_sector[510] != 0x55 || boot_sector[511] != 0xAA {
        return Err("FAT16 Init: Invalid boot sector signature (missing 0xAA55)");
    }

    // Read BIOS Parameter Block (BPB) parameters
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
    let root_dir_size_sectors = ((root_entry_count as u32 * 32) + 511) / 512;
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
    // Set to root
    CURRENT_DIR_CLUSTER = 0;
    Ok(())
}

/// Print disk geometry and volume stats
pub unsafe fn print_disk_info() {
    if let Some(dev) = crate::io::block::get_mounted_device() {
        let sectors = dev.get_size_sectors();
        vga::set_color(vga::Color::LightBlue, vga::Color::Black);
        vga::print_str("Active Drive (");
        vga::print_str(dev.get_name());
        vga::print_str(") Size: ");
        vga::set_color(vga::Color::White, vga::Color::Black);
        vga::print_u64((sectors as u64 * 512) / (1024 * 1024));
        vga::print_str(" MB (");
        vga::print_u64(sectors as u64);
        vga::print_str(" sectors)\n");
    } else {
        vga::set_color(vga::Color::Red, vga::Color::Black);
        vga::print_str("No active block device mounted\n");
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        return;
    }

    let vol_ptr = &raw const VOLUME;
    if let Some(vol) = unsafe { (*vol_ptr).as_ref() } {
        vga::set_color(vga::Color::LightBlue, vga::Color::Black);
        vga::print_str("Filesystem:     ");
        vga::set_color(vga::Color::White, vga::Color::Black);
        vga::print_str("FAT16\n");

        vga::set_color(vga::Color::LightBlue, vga::Color::Black);
        vga::print_str("Cluster Size:   ");
        vga::set_color(vga::Color::White, vga::Color::Black);
        vga::print_u64(vol.sectors_per_cluster as u64 * 512);
        vga::print_str(" bytes (");
        vga::print_u64(vol.sectors_per_cluster as u64);
        vga::print_str(" sectors)\n");

        vga::set_color(vga::Color::LightBlue, vga::Color::Black);
        vga::print_str("Reserved Secs:  ");
        vga::set_color(vga::Color::White, vga::Color::Black);
        vga::print_u64(vol.reserved_sector_count as u64);
        vga::print_str("\n");

        vga::set_color(vga::Color::LightBlue, vga::Color::Black);
        vga::print_str("Root Directory: ");
        vga::set_color(vga::Color::White, vga::Color::Black);
        vga::print_u64(vol.root_entry_count as u64);
        vga::print_str(" entries (start sector: ");
        vga::print_u64(vol.root_dir_start_sector as u64);
        vga::print_str(")\n");
    } else {
        vga::set_color(vga::Color::Yellow, vga::Color::Black);
        vga::print_str("Filesystem:     Not a valid FAT16 partition\n");
    }
    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
}
