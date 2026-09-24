// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! FAT16 volume bootstrap, boot sector validation, and BPB parsing.

use crate::fat::table::{clear_cache, read_sector};
use crate::fat::types::Fat16Volume;

/// Active mounted FAT16 volume metadata descriptor.
pub static mut VOLUME: Option<Fat16Volume> = None;

/// Active current working directory cluster index (0 represents root directory).
pub static mut CURRENT_DIR_CLUSTER: u16 = 0;

/// Initializes the FAT16 filesystem driver by validating the boot sector and reading the BPB.
///
/// # Safety
///
/// Performs block device sector reads and sets global static volume state.
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
