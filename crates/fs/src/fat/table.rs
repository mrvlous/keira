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

//! File Allocation Table (FAT) I/O and sector cache.

pub mod cache {
    #[derive(Copy, Clone)]
    pub struct CacheEntry {
        /// LBA sector number cached in this entry.
        pub sector: u32,
        /// 512-byte raw sector data buffer.
        pub data: [u8; 512],
        /// Validity flag indicating whether entry contains cached data.
        pub valid: bool,
        /// Monotonic timestamp of last read/write access.
        pub last_used: u64,
    }

    pub static mut SECTOR_CACHE: [CacheEntry; 16] = [CacheEntry {
        sector: 0,
        data: [0; 512],
        valid: false,
        last_used: 0,
    }; 16];

    pub static mut CACHE_CLOCK: u64 = 0;

    pub static mut CACHE_HITS: u64 = 0;
    pub static mut CACHE_MISSES: u64 = 0;
    pub static mut CACHE_EVICTIONS: u64 = 0;

    pub fn get_sector_cache_stats() -> (u64, u64, u64, usize) {
        unsafe {
            let active = SECTOR_CACHE.iter().filter(|e| e.valid).count();
            (CACHE_HITS, CACHE_MISSES, CACHE_EVICTIONS, active)
        }
    }

    pub unsafe fn clear_cache() {
        for entry in SECTOR_CACHE.iter_mut() {
            entry.valid = false;
        }
    }
}

pub mod io {
    use super::cache::{
        CacheEntry, CACHE_CLOCK, CACHE_EVICTIONS, CACHE_HITS, CACHE_MISSES, SECTOR_CACHE,
    };
    use keira_io::storage::block::get_mounted_device;

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
}

pub use self::cache::*;
pub use self::io::*;
