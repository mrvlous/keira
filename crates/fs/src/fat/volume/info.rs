// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Disk topology and volume statistics console formatting.

use super::boot::VOLUME;
use keira_io::storage::block::get_mounted_device;
use keira_io::vga;

/// Prints disk geometry, capacity, and mounted volume partition statistics to VGA console.
///
/// # Safety
///
/// Reads static volume pointers and interacts directly with VGA driver registers.
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
    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
}
