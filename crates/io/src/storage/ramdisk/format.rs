// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Dynamic RAM disk frame allocation and synthetic FAT16 filesystem formatter.

use super::device::{MAX_RAMDISK_FRAMES, RAM_DEVICE};
use crate::storage::block::{register_device, BlockDevice};
use keira_mem::pmm;

/// Allocates physical frames for the system RAM disk, formats a FAT16 filesystem, and registers the device.
pub fn create_ramdisk(size_kb: usize) -> Result<(), &'static str> {
    let frames_needed = (size_kb * 1024 + 4095) / 4096;
    if frames_needed == 0 || frames_needed > MAX_RAMDISK_FRAMES {
        return Err("Ramdisk: Invalid size or exceeds MAX_RAMDISK_FRAMES");
    }

    free_current_ramdisk();

    unsafe {
        RAM_DEVICE.frame_count = 0;
        for _ in 0..frames_needed {
            let frame = pmm::alloc_frame().ok_or("Ramdisk: Out of physical memory")?;
            core::ptr::write_bytes(frame as *mut u8, 0, 4096);
            RAM_DEVICE.frames[RAM_DEVICE.frame_count] = frame;
            RAM_DEVICE.frame_count += 1;
        }

        RAM_DEVICE.size_sectors = (frames_needed * 8) as u32;

        format_fat16(&*core::ptr::addr_of!(RAM_DEVICE))?;

        let _ = register_device(&*core::ptr::addr_of!(RAM_DEVICE));
    }

    Ok(())
}

/// Deallocates all physical frames currently assigned to the system RAM disk.
pub fn free_current_ramdisk() {
    unsafe {
        for i in 0..RAM_DEVICE.frame_count {
            if RAM_DEVICE.frames[i] != 0 {
                pmm::free_frame(RAM_DEVICE.frames[i]);
                RAM_DEVICE.frames[i] = 0;
            }
        }
        RAM_DEVICE.frame_count = 0;
        RAM_DEVICE.size_sectors = 0;
    }
}

fn format_fat16(device: &dyn BlockDevice) -> Result<(), &'static str> {
    let total_sectors = device.get_size_sectors();
    let sectors_per_cluster: u8 = if total_sectors < 8192 {
        4
    } else if total_sectors < 16384 {
        8
    } else {
        16
    };

    let reserved_sectors: u16 = 4;
    let num_fats: u8 = 2;
    let root_entries: u16 = 512;
    let root_dir_sectors = ((root_entries * 32) + 511) / 512;

    let data_sectors = total_sectors
        .saturating_sub(reserved_sectors as u32)
        .saturating_sub(root_dir_sectors as u32);
    let total_clusters = data_sectors / (sectors_per_cluster as u32);

    let bytes_per_fat = total_clusters * 2;
    let sectors_per_fat = ((bytes_per_fat + 511) / 512) as u16;

    let mut boot_sec = [0u8; 512];
    boot_sec[0] = 0xEB;
    boot_sec[1] = 0x3C;
    boot_sec[2] = 0x90;
    boot_sec[3..11].copy_from_slice(b"KEIRAOS ");
    boot_sec[11] = 0x00;
    boot_sec[12] = 0x02;
    boot_sec[13] = sectors_per_cluster;
    boot_sec[14] = (reserved_sectors & 0xFF) as u8;
    boot_sec[15] = ((reserved_sectors >> 8) & 0xFF) as u8;
    boot_sec[16] = num_fats;
    boot_sec[17] = (root_entries & 0xFF) as u8;
    boot_sec[18] = ((root_entries >> 8) & 0xFF) as u8;
    boot_sec[19] = (total_sectors & 0xFF) as u8;
    boot_sec[20] = ((total_sectors >> 8) & 0xFF) as u8;
    boot_sec[21] = 0xF8;
    boot_sec[22] = (sectors_per_fat & 0xFF) as u8;
    boot_sec[23] = ((sectors_per_fat >> 8) & 0xFF) as u8;
    boot_sec[24] = 18;
    boot_sec[25] = 0;
    boot_sec[26] = 2;
    boot_sec[27] = 0;
    boot_sec[28] = 0;
    boot_sec[29] = 0;
    boot_sec[30] = 0;
    boot_sec[31] = 0;
    boot_sec[32] = 0;
    boot_sec[33] = 0;
    boot_sec[34] = 0;
    boot_sec[35] = 0;

    boot_sec[36] = 0x80;
    boot_sec[37] = 0x00;
    boot_sec[38] = 0x29;
    boot_sec[39] = 0x78;
    boot_sec[40] = 0x56;
    boot_sec[41] = 0x34;
    boot_sec[42] = 0x12;
    boot_sec[43..54].copy_from_slice(b"KEIRA RAM  ");
    boot_sec[54..62].copy_from_slice(b"FAT16   ");

    boot_sec[510] = 0x55;
    boot_sec[511] = 0xAA;

    device.write_sector(0, &boot_sec)?;

    let zero_sec = [0u8; 512];
    for s in 1..reserved_sectors {
        device.write_sector(s as u32, &zero_sec)?;
    }

    let mut fat_start_sec = [0u8; 512];
    fat_start_sec[0] = 0xF8;
    fat_start_sec[1] = 0xFF;
    fat_start_sec[2] = 0xFF;
    fat_start_sec[3] = 0xFF;

    for fat in 0..num_fats as u32 {
        let base = reserved_sectors as u32 + (fat * sectors_per_fat as u32);
        device.write_sector(base, &fat_start_sec)?;
        for s in 1..sectors_per_fat {
            device.write_sector(base + s as u32, &zero_sec)?;
        }
    }

    let root_start = reserved_sectors as u32 + (num_fats as u32 * sectors_per_fat as u32);
    for s in 0..root_dir_sectors {
        device.write_sector(root_start + s as u32, &zero_sec)?;
    }

    Ok(())
}
