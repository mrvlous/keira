// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Keira Kernel: IDE Disk Driver (PIO Mode)
//!
//! Provides support for reading and writing 512-byte sectors from/to the
//! primary master ATA IDE hard drive using LBA28 addressing.

const IDE_DATA: u16 = 0x1F0;
const _IDE_ERROR: u16 = 0x1F1;
const IDE_SECCOUNT: u16 = 0x1F2;
const IDE_LBA_LOW: u16 = 0x1F3;
const IDE_LBA_MID: u16 = 0x1F4;
const IDE_LBA_HIGH: u16 = 0x1F5;
const IDE_DEV_SEL: u16 = 0x1F6;
const IDE_STATUS: u16 = 0x1F7;
const IDE_COMMAND: u16 = 0x1F7;

// IDE Status Register Bits
const STATUS_ERR: u8 = 1 << 0;
const STATUS_DRQ: u8 = 1 << 3;
const STATUS_BSY: u8 = 1 << 7;

// Assembly Port I/O Helpers
unsafe fn outb(port: u16, val: u8) {
    core::arch::asm!("out dx, al", in("dx") port, in("al") val, options(nomem, nostack, preserves_flags));
}

unsafe fn inb(port: u16) -> u8 {
    let res: u8;
    core::arch::asm!("in al, dx", out("al") res, in("dx") port, options(nomem, nostack, preserves_flags));
    res
}

unsafe fn inw(port: u16) -> u16 {
    let res: u16;
    core::arch::asm!("in ax, dx", out("ax") res, in("dx") port, options(nomem, nostack, preserves_flags));
    res
}

unsafe fn outw(port: u16, val: u16) {
    core::arch::asm!("out dx, ax", in("dx") port, in("ax") val, options(nomem, nostack, preserves_flags));
}

/// Wait for the IDE controller to become ready (BSY cleared)
unsafe fn ide_wait_ready() -> Result<(), &'static str> {
    let mut timeout = 100_000;
    while (inb(IDE_STATUS) & STATUS_BSY) != 0 {
        timeout -= 1;
        if timeout == 0 {
            return Err("IDE controller timeout: BSY stuck high");
        }
    }
    Ok(())
}

/// Identify the Primary Master IDE drive and return its size in sectors (LBA28)
pub unsafe fn identify() -> Result<u32, &'static str> {
    // 1. Select the master drive
    outb(IDE_DEV_SEL, 0xA0);

    // 2. Clear sector count and LBA registers
    outb(IDE_SECCOUNT, 0);
    outb(IDE_LBA_LOW, 0);
    outb(IDE_LBA_MID, 0);
    outb(IDE_LBA_HIGH, 0);

    // 3. Send IDENTIFY command
    outb(IDE_COMMAND, 0xEC);

    // 4. Check if drive exists
    let status = inb(IDE_STATUS);
    if status == 0 {
        return Err("IDE: Drive does not exist");
    }

    // 5. Wait for BSY to clear
    ide_wait_ready()?;

    // 6. Check if it is a non-ATA drive
    let lba_mid = inb(IDE_LBA_MID);
    let lba_high = inb(IDE_LBA_HIGH);
    if lba_mid != 0 || lba_high != 0 {
        return Err("IDE: Non-ATA drive detected");
    }

    // 7. Wait for DRQ or ERR
    let mut timeout = 100_000;
    loop {
        let stat = inb(IDE_STATUS);
        if (stat & STATUS_ERR) != 0 {
            return Err("IDE: Identify failed with error status");
        }
        if (stat & STATUS_DRQ) != 0 {
            break;
        }
        timeout -= 1;
        if timeout == 0 {
            return Err("IDE: Timeout waiting for identify DRQ");
        }
    }

    // 8. Read 256 words (512 bytes) of identification data
    let mut id_data = [0u16; 256];
    for i in 0..256 {
        id_data[i] = inw(IDE_DATA);
    }

    // Word 60 and 61 contain the total user addressable sectors for LBA28
    let sectors = (id_data[60] as u32) | ((id_data[61] as u32) << 16);

    Ok(sectors)
}

/// Read a 512-byte sector from the IDE primary master drive using LBA28
pub unsafe fn read_sector(lba: u32, buffer: &mut [u8; 512]) -> Result<(), &'static str> {
    if lba > 0x0FFFFFFF {
        return Err("IDE Read: LBA address exceeds 28-bit limit");
    }

    // 1. Wait for controller ready
    ide_wait_ready()?;

    // 2. Select drive and LBA bits 24-27
    outb(IDE_DEV_SEL, 0xE0 | (((lba >> 24) & 0x0F) as u8));

    // 3. Write Sector Count and LBA bits 0-23
    outb(IDE_SECCOUNT, 1);
    outb(IDE_LBA_LOW, (lba & 0xFF) as u8);
    outb(IDE_LBA_MID, ((lba >> 8) & 0xFF) as u8);
    outb(IDE_LBA_HIGH, ((lba >> 16) & 0xFF) as u8);

    // 4. Send READ SECTORS command
    outb(IDE_COMMAND, 0x20);

    // 5. Wait for DRQ
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
            return Err("IDE Read: Timeout waiting for data transfer (DRQ)");
        }
    }

    // 6. Read 256 16-bit words (512 bytes) into buffer
    let buf_u16 = buffer.as_mut_ptr() as *mut u16;
    for i in 0..256 {
        *buf_u16.add(i) = inw(IDE_DATA);
    }

    Ok(())
}

/// Write a 512-byte sector to the IDE primary master drive using LBA28
pub unsafe fn write_sector(lba: u32, buffer: &[u8; 512]) -> Result<(), &'static str> {
    if lba > 0x0FFFFFFF {
        return Err("IDE Write: LBA address exceeds 28-bit limit");
    }

    // 1. Wait for controller ready
    ide_wait_ready()?;

    // 2. Select drive and LBA bits 24-27
    outb(IDE_DEV_SEL, 0xE0 | (((lba >> 24) & 0x0F) as u8));

    // 3. Write Sector Count and LBA bits 0-23
    outb(IDE_SECCOUNT, 1);
    outb(IDE_LBA_LOW, (lba & 0xFF) as u8);
    outb(IDE_LBA_MID, ((lba >> 8) & 0xFF) as u8);
    outb(IDE_LBA_HIGH, ((lba >> 16) & 0xFF) as u8);

    // 4. Send WRITE SECTORS command
    outb(IDE_COMMAND, 0x30);

    // 5. Wait for DRQ (ready to accept data)
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

    // 6. Write 256 16-bit words (512 bytes) from buffer
    let buf_u16 = buffer.as_ptr() as *const u16;
    for i in 0..256 {
        outw(IDE_DATA, *buf_u16.add(i));
    }

    // 7. Wait for BSY to clear (flushed to disk)
    ide_wait_ready()?;

    Ok(())
}

use crate::io::block::BlockDevice;

pub struct IdeBlockDevice {
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
