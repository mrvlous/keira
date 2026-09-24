// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! AHCI SATA Block Device structure implementation.

use super::driver::{flush_dma_cache, sata_dma_transfer, sata_flush_cache, SECTOR_BUF_PHYS};
use crate::storage::block::BlockDevice;

/// AHCI SATA Block Device implementation.
pub struct AhciBlockDevice {
    /// Controller port index attached to this drive.
    pub port_num: usize,
    /// Total capacity in 512-byte sectors.
    pub size_sectors: u32,
}

impl BlockDevice for AhciBlockDevice {
    fn read_sector(&self, sector: u32, buffer: &mut [u8; 512]) -> Result<(), &'static str> {
        unsafe {
            sata_dma_transfer(self.port_num, sector, false)?;
            let src = SECTOR_BUF_PHYS as *const u8;
            core::ptr::copy_nonoverlapping(src, buffer.as_mut_ptr(), 512);
            Ok(())
        }
    }

    fn write_sector(&self, sector: u32, buffer: &[u8; 512]) -> Result<(), &'static str> {
        unsafe {
            let dst = SECTOR_BUF_PHYS as *mut u8;
            core::ptr::copy_nonoverlapping(buffer.as_ptr(), dst, 512);
            sata_dma_transfer(self.port_num, sector, true)?;
            flush_dma_cache();
            Ok(())
        }
    }

    fn get_size_sectors(&self) -> u32 {
        self.size_sectors
    }

    fn get_name(&self) -> &'static str {
        "ahci0"
    }

    fn flush(&self) -> Result<(), &'static str> {
        unsafe { sata_flush_cache(self.port_num) }
    }
}

/// Global detected AHCI SATA block device instance.
pub static mut AHCI_DEVICE: Option<AhciBlockDevice> = None;
