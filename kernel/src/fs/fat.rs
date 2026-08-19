// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

pub mod cluster;
pub mod dir;
pub mod file;
pub mod path;
pub mod types;
pub mod volume;

pub use dir::{
    change_directory, create_dir, find_matches, get_dir_cluster, list_files, list_files_in_dir,
};
pub use file::{
    append_file_content, cat_file, create_file, read_file_content, remove_entry, write_file_content,
};
pub use path::{filename_to_8_3, find_entry, format_filename, resolve_path};
pub use types::{DirectoryEntry, Fat16Volume, FoundEntry};
pub use volume::{init, print_disk_info};

pub static mut VOLUME: Option<types::Fat16Volume> = None;
// 0 = Root Directory
pub static mut CURRENT_DIR_CLUSTER: u16 = 0;

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

/// Clear/invalidate the sector cache
pub unsafe fn clear_cache() {
    for i in 0..16 {
        SECTOR_CACHE[i].valid = false;
    }
}

/// Local helper to read a sector from the currently mounted block device, using a sector cache
pub unsafe fn read_sector(sector: u32, buffer: &mut [u8; 512]) -> Result<(), &'static str> {
    // 1. Search in cache
    for i in 0..16 {
        if SECTOR_CACHE[i].valid && SECTOR_CACHE[i].sector == sector {
            SECTOR_CACHE[i].last_used = CACHE_CLOCK;
            CACHE_CLOCK += 1;
            buffer.copy_from_slice(&SECTOR_CACHE[i].data);
            return Ok(());
        }
    }

    // 2. Cache miss: Read from device
    let mut dev_data = [0u8; 512];
    if let Some(dev) = crate::io::block::get_mounted_device() {
        dev.read_sector(sector, &mut dev_data)?;
    } else {
        return Err("FAT16 Error: No block device mounted");
    }

    // 3. Find slot to insert (either invalid or least recently used)
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

    // 4. Update the selected cache entry
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

/// Local helper to write a sector to the currently mounted block device, using write-through cache
pub unsafe fn write_sector(sector: u32, buffer: &[u8; 512]) -> Result<(), &'static str> {
    // 1. Update cache entry if present
    for i in 0..16 {
        if SECTOR_CACHE[i].valid && SECTOR_CACHE[i].sector == sector {
            SECTOR_CACHE[i].data.copy_from_slice(buffer);
            SECTOR_CACHE[i].last_used = CACHE_CLOCK;
            CACHE_CLOCK += 1;
        }
    }

    // 2. Write-through to the physical/virtual device
    if let Some(dev) = crate::io::block::get_mounted_device() {
        dev.write_sector(sector, buffer)?;
    } else {
        return Err("FAT16 Error: No block device mounted");
    }

    Ok(())
}

/// Flush dirty sectors to mounted block storage device
pub unsafe fn flush_dirty_sectors() -> Result<usize, &'static str> {
    let mut count = 0usize;
    if let Some(dev) = crate::io::block::get_mounted_device() {
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
