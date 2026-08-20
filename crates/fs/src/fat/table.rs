// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! 16-slot LRU cached sector I/O reader and write-through cache engine for FAT16.

use keira_io::storage::block::get_mounted_device;

#[derive(Copy, Clone)]
struct CacheEntry {
    sector: u32,
    data: [u8; 512],
    valid: bool,
    last_used: u64,
}

static mut SECTOR_CACHE: [CacheEntry; 16] = [CacheEntry {
    sector: 0,
    data: [0; 512],
    valid: false,
    last_used: 0,
}; 16];

static mut CACHE_CLOCK: u64 = 0;

/// Invalidate all entries in the sector cache.
pub unsafe fn clear_cache() {
    for i in 0..16 {
        SECTOR_CACHE[i].valid = false;
    }
}

/// Read a 512-byte sector from the currently mounted block device, utilizing the LRU cache.
pub unsafe fn read_sector(sector: u32, buffer: &mut [u8; 512]) -> Result<(), &'static str> {
    for i in 0..16 {
        if SECTOR_CACHE[i].valid && SECTOR_CACHE[i].sector == sector {
            SECTOR_CACHE[i].last_used = CACHE_CLOCK;
            CACHE_CLOCK += 1;
            buffer.copy_from_slice(&SECTOR_CACHE[i].data);
            return Ok(());
        }
    }

    let mut dev_data = [0u8; 512];
    if let Some(dev) = get_mounted_device() {
        dev.read_sector(sector, &mut dev_data)?;
    } else {
        return Err("FAT16 Error: No block device mounted");
    }

    let mut best_index = 0;
    let mut min_lru = u64::MAX;

    for i in 0..16 {
        if !SECTOR_CACHE[i].valid {
            best_index = i;
            break;
        }
        if SECTOR_CACHE[i].last_used < min_lru {
            min_lru = SECTOR_CACHE[i].last_used;
            best_index = i;
        }
    }

    SECTOR_CACHE[best_index] = CacheEntry {
        sector,
        data: dev_data,
        valid: true,
        last_used: CACHE_CLOCK,
    };
    CACHE_CLOCK += 1;

    buffer.copy_from_slice(&dev_data);
    Ok(())
}

/// Write a 512-byte sector to the mounted block device using write-through caching.
pub unsafe fn write_sector(sector: u32, buffer: &[u8; 512]) -> Result<(), &'static str> {
    for i in 0..16 {
        if SECTOR_CACHE[i].valid && SECTOR_CACHE[i].sector == sector {
            SECTOR_CACHE[i].data.copy_from_slice(buffer);
            SECTOR_CACHE[i].last_used = CACHE_CLOCK;
            CACHE_CLOCK += 1;
        }
    }

    if let Some(dev) = get_mounted_device() {
        dev.write_sector(sector, buffer)?;
    } else {
        return Err("FAT16 Error: No block device mounted");
    }

    Ok(())
}

/// Flush dirty sectors to mounted block storage device.
pub unsafe fn flush_dirty_sectors() -> Result<usize, &'static str> {
    let mut count = 0usize;
    if let Some(dev) = get_mounted_device() {
        for i in 0..16 {
            if SECTOR_CACHE[i].valid {
                let sec = SECTOR_CACHE[i].sector;
                let data = SECTOR_CACHE[i].data;
                dev.write_sector(sec, &data)?;
                count += 1;
            }
        }
    }
    Ok(count)
}
