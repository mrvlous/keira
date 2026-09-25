// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Cached sector I/O reader and write-through cache engine for FAT16.

use super::cache::{
    CacheEntry, CACHE_CLOCK, CACHE_EVICTIONS, CACHE_HITS, CACHE_MISSES, SECTOR_CACHE,
};
use keira_io::storage::block::get_mounted_device;

/// Reads a 512-byte sector from the currently mounted block device, utilizing the LRU cache.
///
/// # Safety
///
/// Accesses the global mutable sector cache and invokes block driver I/O.
pub unsafe fn read_sector(sector: u32, buffer: &mut [u8; 512]) -> Result<(), &'static str> {
    for entry in SECTOR_CACHE.iter_mut() {
        if entry.valid && entry.sector == sector {
            entry.last_used = CACHE_CLOCK;
            CACHE_CLOCK += 1;
            CACHE_HITS += 1;
            buffer.copy_from_slice(&entry.data);
            return Ok(());
        }
    }

    CACHE_MISSES += 1;
    let mut dev_data = [0u8; 512];
    if let Some(dev) = get_mounted_device() {
        dev.read_sector(sector, &mut dev_data)?;
    } else {
        return Err("FAT16 Error: No block device mounted");
    }

    let mut best_index = 0;
    let mut min_lru = u64::MAX;

    for (i, entry) in SECTOR_CACHE.iter().enumerate() {
        if !entry.valid {
            best_index = i;
            break;
        }
        if entry.last_used < min_lru {
            min_lru = entry.last_used;
            best_index = i;
        }
    }

    if SECTOR_CACHE[best_index].valid {
        CACHE_EVICTIONS += 1;
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

/// Writes a 512-byte sector to the mounted block device using write-through caching.
///
/// # Safety
///
/// Accesses the global mutable sector cache and invokes block driver I/O.
pub unsafe fn write_sector(sector: u32, buffer: &[u8; 512]) -> Result<(), &'static str> {
    for entry in SECTOR_CACHE.iter_mut() {
        if entry.valid && entry.sector == sector {
            entry.data.copy_from_slice(buffer);
            entry.last_used = CACHE_CLOCK;
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

/// Flushes dirty sectors to mounted block storage device and triggers hardware cache barrier.
///
/// # Safety
///
/// Accesses the global mutable sector cache and invokes block driver flush operations.
pub unsafe fn flush_dirty_sectors() -> Result<usize, &'static str> {
    let mut count = 0usize;
    if let Some(dev) = get_mounted_device() {
        for entry in SECTOR_CACHE.iter() {
            if entry.valid {
                let sec = entry.sector;
                let data = entry.data;
                dev.write_sector(sec, &data)?;
                count += 1;
            }
        }
        dev.flush()?;
    }
    Ok(count)
}
