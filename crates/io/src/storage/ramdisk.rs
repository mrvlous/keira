// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! In-memory RAM disk block device emulator and FAT16 initialization formatter.

use super::block::{register_device, BlockDevice};
use keira_mem::pmm;

const MAX_RAMDISK_FRAMES: usize = 1024;

/// In-memory RAM disk block device backed by allocated physical pages.
pub struct RamBlockDevice {
    pub name: &'static str,
    pub size_sectors: u32,
    pub frames: [u64; MAX_RAMDISK_FRAMES],
    pub frame_count: usize,
}

impl BlockDevice for RamBlockDevice {
    fn read_sector(&self, sector: u32, buffer: &mut [u8; 512]) -> Result<(), &'static str> {
        if sector >= self.size_sectors {
            return Err("Ramdisk Read: Out of bounds");
        }

        let frame_idx = (sector / 8) as usize;
        let sector_offset = ((sector % 8) * 512) as usize;

        if frame_idx >= self.frame_count {
            return Err("Ramdisk Read: Internal frame index error");
        }

        let frame_addr = self.frames[frame_idx];
        if frame_addr == 0 {
            return Err("Ramdisk Read: Invalid frame address");
        }

        unsafe {
            let src = (frame_addr + sector_offset as u64) as *const u8;
            core::ptr::copy_nonoverlapping(src, buffer.as_mut_ptr(), 512);
        }

        Ok(())
    }

    fn write_sector(&self, sector: u32, buffer: &[u8; 512]) -> Result<(), &'static str> {
        if sector >= self.size_sectors {
            return Err("Ramdisk Write: Out of bounds");
        }

        let frame_idx = (sector / 8) as usize;
        let sector_offset = ((sector % 8) * 512) as usize;

        if frame_idx >= self.frame_count {
            return Err("Ramdisk Write: Internal frame index error");
        }

        let frame_addr = self.frames[frame_idx];
        if frame_addr == 0 {
            return Err("Ramdisk Write: Invalid frame address");
        }

        unsafe {
            let dest = (frame_addr + sector_offset as u64) as *mut u8;
            core::ptr::copy_nonoverlapping(buffer.as_ptr(), dest, 512);
        }

        Ok(())
    }

    fn get_size_sectors(&self) -> u32 {
        self.size_sectors
    }

    fn get_name(&self) -> &'static str {
        self.name
    }
}

pub static mut RAM_DEVICE: RamBlockDevice = RamBlockDevice {
    name: "ram0",
    size_sectors: 0,
    frames: [0; MAX_RAMDISK_FRAMES],
    frame_count: 0,
};

/// Initialize a dynamic Ramdisk of a specified size (in Kilobytes).
pub fn create_ramdisk(size_kb: u32) -> Result<(), &'static str> {
    if size_kb == 0 {
        return Err("Ramdisk size must be greater than 0");
    }

    let required_frames = size_kb.div_ceil(4);
    if required_frames as usize > MAX_RAMDISK_FRAMES {
        return Err("Requested Ramdisk size exceeds max limit (4MB)");
    }

    unsafe {
        free_current_ramdisk();

        let mut allocated = 0;
        let mut success = true;

        for i in 0..required_frames as usize {
            if let Some(frame) = pmm::alloc_frame() {
                RAM_DEVICE.frames[i] = frame;
                allocated += 1;
            } else {
                success = false;
                break;
            }
        }

        if !success {
            for i in 0..allocated {
                pmm::free_frame(RAM_DEVICE.frames[i]);
                RAM_DEVICE.frames[i] = 0;
            }
            return Err("Ramdisk Init: Out of physical memory frames");
        }

        RAM_DEVICE.frame_count = required_frames as usize;
        RAM_DEVICE.size_sectors = required_frames * 8;

        format_fat16(&*core::ptr::addr_of!(RAM_DEVICE))?;
        register_device(&*core::ptr::addr_of!(RAM_DEVICE))?;
    }

    Ok(())
}

/// Helper to free current Ramdisk allocations.
pub unsafe fn free_current_ramdisk() {
    for i in 0..RAM_DEVICE.frame_count {
        if RAM_DEVICE.frames[i] != 0 {
            pmm::free_frame(RAM_DEVICE.frames[i]);
            RAM_DEVICE.frames[i] = 0;
        }
    }
    RAM_DEVICE.frame_count = 0;
    RAM_DEVICE.size_sectors = 0;
}

fn format_fat16(device: &dyn BlockDevice) -> Result<(), &'static str> {
    let total_sectors = device.get_size_sectors();
    let sectors_per_cluster = 2;
    let reserved_sectors = 4;
    let num_fats = 2;
    let root_dir_sectors = 32;

    let total_clusters = total_sectors / sectors_per_cluster as u32;
    let sectors_per_fat = (total_clusters * 2).div_ceil(512);

    let mut boot_sec = [0u8; 512];
    boot_sec[0] = 0xEB;
    boot_sec[1] = 0x3C;
    boot_sec[2] = 0x90;
    let oem = b"KEIRA   ";
    boot_sec[3..11].copy_from_slice(oem);

    boot_sec[11] = 0x00;
    boot_sec[12] = 0x02;
    boot_sec[13] = sectors_per_cluster;
    boot_sec[14] = (reserved_sectors & 0xFF) as u8;
    boot_sec[15] = ((reserved_sectors >> 8) & 0xFF) as u8;
    boot_sec[16] = num_fats;
    boot_sec[17] = 0x00;
    boot_sec[18] = 0x02;
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
        let base = reserved_sectors as u32 + (fat * sectors_per_fat);
        device.write_sector(base, &fat_start_sec)?;
        for s in 1..sectors_per_fat {
            device.write_sector(base + s, &zero_sec)?;
        }
    }

    let root_start = reserved_sectors as u32 + (num_fats as u32 * sectors_per_fat);
    for s in 0..root_dir_sectors {
        device.write_sector(root_start + s, &zero_sec)?;
    }

    Ok(())
}
