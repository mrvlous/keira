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

//! Integrated Drive Electronics (IDE / Parallel ATA) storage driver.

pub mod device {
    use super::driver::{read_sector, write_sector};
    use crate::storage::block::BlockDevice;
    
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
    
    pub static mut IDE_DEVICE: IdeBlockDevice = IdeBlockDevice { size_sectors: 0 };
}

pub mod driver {
    use keira_arch::cpu::{inb, inw, outb, outw};
    
    pub const IDE_DATA: u16 = 0x1F0;
    pub const _IDE_ERROR: u16 = 0x1F1;
    pub const IDE_SECCOUNT: u16 = 0x1F2;
    pub const IDE_LBA_LOW: u16 = 0x1F3;
    pub const IDE_LBA_MID: u16 = 0x1F4;
    pub const IDE_LBA_HIGH: u16 = 0x1F5;
    pub const IDE_DEV_SEL: u16 = 0x1F6;
    pub const IDE_STATUS: u16 = 0x1F7;
    pub const IDE_COMMAND: u16 = 0x1F7;
    
    pub const STATUS_ERR: u8 = 1 << 0;
    pub const STATUS_DRQ: u8 = 1 << 3;
    pub const STATUS_BSY: u8 = 1 << 7;
    
    pub unsafe fn ide_wait_ready() -> Result<(), &'static str> {
        let mut timeout = 100_000;
        while (inb(IDE_STATUS) & STATUS_BSY) != 0 {
            timeout -= 1;
            if timeout == 0 {
                return Err("IDE controller timeout: BSY stuck high");
            }
        }
        Ok(())
    }
    
    pub unsafe fn identify() -> Result<u32, &'static str> {
        outb(IDE_DEV_SEL, 0xA0);
        outb(IDE_SECCOUNT, 0);
        outb(IDE_LBA_LOW, 0);
        outb(IDE_LBA_MID, 0);
        outb(IDE_LBA_HIGH, 0);
        outb(IDE_COMMAND, 0xEC);
    
        let status = inb(IDE_STATUS);
        if status == 0 {
            return Err("IDE: Drive does not exist");
        }
    
        ide_wait_ready()?;
    
        let lba_mid = inb(IDE_LBA_MID);
        let lba_high = inb(IDE_LBA_HIGH);
        if lba_mid != 0 || lba_high != 0 {
            return Err("IDE: Non-ATA drive detected");
        }
    
        let mut timeout = 100_000;
        loop {
            let stat = inb(IDE_STATUS);
            if (stat & STATUS_ERR) != 0 {
                return Err("IDE: Error during IDENTIFY");
            }
            if (stat & STATUS_DRQ) != 0 {
                break;
            }
            timeout -= 1;
            if timeout == 0 {
                return Err("IDE: Timeout waiting for DRQ");
            }
        }
    
        let mut data = [0u16; 256];
        for word in data.iter_mut() {
            *word = inw(IDE_DATA);
        }
    
        let sectors_28 = (data[60] as u32) | ((data[61] as u32) << 16);
        Ok(sectors_28)
    }
    
    pub unsafe fn read_sector(lba: u32, buffer: &mut [u8; 512]) -> Result<(), &'static str> {
        if lba > 0x0FFFFFFF {
            return Err("IDE Read: LBA address exceeds 28-bit limit");
        }
    
        ide_wait_ready()?;
        outb(IDE_DEV_SEL, 0xE0 | (((lba >> 24) & 0x0F) as u8));
        outb(IDE_SECCOUNT, 1);
        outb(IDE_LBA_LOW, (lba & 0xFF) as u8);
        outb(IDE_LBA_MID, ((lba >> 8) & 0xFF) as u8);
        outb(IDE_LBA_HIGH, ((lba >> 16) & 0xFF) as u8);
        outb(IDE_COMMAND, 0x20);
    
        let mut timeout = 100_000;
        loop {
            let stat = inb(IDE_STATUS);
            if (stat & STATUS_ERR) != 0 {
                return Err("IDE Read: Controller error flag set");
            }
            if (stat & STATUS_DRQ) != 0 {
                break;
            }
            timeout -= 1;
            if timeout == 0 {
                return Err("IDE Read: Timeout waiting for DRQ");
            }
        }
    
        let buf_u16 = buffer.as_mut_ptr() as *mut u16;
        for i in 0..256 {
            *buf_u16.add(i) = inw(IDE_DATA);
        }
    
        Ok(())
    }
    
    pub unsafe fn write_sector(lba: u32, buffer: &[u8; 512]) -> Result<(), &'static str> {
        if lba > 0x0FFFFFFF {
            return Err("IDE Write: LBA address exceeds 28-bit limit");
        }
    
        ide_wait_ready()?;
        outb(IDE_DEV_SEL, 0xE0 | (((lba >> 24) & 0x0F) as u8));
        outb(IDE_SECCOUNT, 1);
        outb(IDE_LBA_LOW, (lba & 0xFF) as u8);
        outb(IDE_LBA_MID, ((lba >> 8) & 0xFF) as u8);
        outb(IDE_LBA_HIGH, ((lba >> 16) & 0xFF) as u8);
        outb(IDE_COMMAND, 0x30);
    
        let mut timeout = 100_000;
        loop {
            let stat = inb(IDE_STATUS);
            if (stat & STATUS_ERR) != 0 {
                return Err("IDE Write: Controller error flag set before write");
            }
            if (stat & STATUS_DRQ) != 0 {
                break;
            }
            timeout -= 1;
            if timeout == 0 {
                return Err("IDE Write: Timeout waiting for DRQ before write");
            }
        }
    
        let buf_u16 = buffer.as_ptr() as *const u16;
        for i in 0..256 {
            outw(IDE_DATA, *buf_u16.add(i));
        }
    
        ide_wait_ready()?;
        Ok(())
    }
}

pub use self::device::*;
pub use self::driver::*;

#[cfg(test)]
mod tests {
    use super::device::IdeBlockDevice;
    use super::driver::read_sector;
    use crate::storage::block::BlockDevice;
    
    #[test]
    fn test_ide_device_metadata() {
        let dev = IdeBlockDevice { size_sectors: 4096 };
        assert_eq!(dev.get_name(), "ide0");
        assert_eq!(dev.get_size_sectors(), 4096);
    }
    
    #[test]
    fn test_ide_lba28_out_of_bounds() {
        let mut buf = [0u8; 512];
        unsafe {
            let res = read_sector(0x1000_0000, &mut buf);
            assert!(res.is_err());
        }
    }
}
