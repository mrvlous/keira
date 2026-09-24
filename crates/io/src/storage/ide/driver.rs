// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Primary Master ATA IDE I/O port command routines using LBA28.

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

/// Polls the IDE controller status register until the busy flag is cleared.
///
/// # Safety
///
/// Direct port I/O to ATA command ports.
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

/// Identifies the Primary Master IDE drive and returns total sector count.
///
/// # Safety
///
/// Direct port I/O to ATA command ports.
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

/// Reads a single 512-byte sector via 28-bit LBA addressing.
///
/// # Safety
///
/// Direct port I/O to ATA data and command ports.
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

/// Writes a single 512-byte sector via 28-bit LBA addressing.
///
/// # Safety
///
/// Direct port I/O to ATA data and command ports.
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
