// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! In-memory RAM disk block device implementation backed by physical memory frames.

use crate::storage::block::BlockDevice;

/// Maximum number of 4 KiB physical memory frames allocated to a RAM disk (4 MiB capacity).
pub const MAX_RAMDISK_FRAMES: usize = 1024;

/// In-memory RAM disk block device backed by allocated physical pages.
pub struct RamBlockDevice {
    /// Device identifier string.
    pub name: &'static str,
    /// Total device capacity expressed in 512-byte sectors.
    pub size_sectors: u32,
    /// Physical base addresses of allocated 4 KiB frames.
    pub frames: [u64; MAX_RAMDISK_FRAMES],
    /// Count of populated physical frames in `frames`.
    pub frame_count: usize,
}

impl Default for RamBlockDevice {
    fn default() -> Self {
        Self::new()
    }
}

impl RamBlockDevice {
    /// Constructs a blank, unallocated RAM disk device descriptor.
    pub const fn new() -> Self {
        Self {
            name: "ram0",
            size_sectors: 0,
            frames: [0; MAX_RAMDISK_FRAMES],
            frame_count: 0,
        }
    }

    /// Block size in bytes.
    pub const fn block_size(&self) -> u32 {
        512
    }

    /// Block count.
    pub const fn block_count(&self) -> u32 {
        self.size_sectors
    }
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
            let dst = (frame_addr + sector_offset as u64) as *mut u8;
            core::ptr::copy_nonoverlapping(buffer.as_ptr(), dst, 512);
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

/// Global system RAM disk block device instance.
pub static mut RAM_DEVICE: RamBlockDevice = RamBlockDevice::new();
