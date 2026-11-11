// SPDX-License-Identifier: GPL-2.0-only

//! ATA IDE Block Device structure implementation.

use super::driver::{read_sector, write_sector};
use crate::storage::block::BlockDevice;

/// ATA IDE Block Device implementation.
pub struct IdeBlockDevice {
    /// Total capacity of the detected drive in 512-byte sectors.
    pub size_sectors: u32,
}

impl BlockDevice for IdeBlockDevice {
    fn read_sector(&self, sector: u32, buffer: &mut [u8; 512]) -> Result<(), &'static str> {
        unsafe { read_sector(sector, buffer) }
    }

    fn write_sector(&self, sector: u32, buffer: &[u8; 512]) -> Result<(), &'static str> {
        unsafe { write_sector(sector, buffer) }
    }

    fn get_size_sectors(&self) -> u32 {
        self.size_sectors
    }

    fn get_name(&self) -> &'static str {
        "ide0"
    }
}

/// Global ATA Primary Master IDE device instance.
pub static mut IDE_DEVICE: IdeBlockDevice = IdeBlockDevice { size_sectors: 0 };
