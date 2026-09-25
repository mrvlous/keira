// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! 16-slot LRU sector cache storage and invalidation engine.

/// Internal cache entry holding a single 512-byte block device sector.
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

/// Global 16-slot Least Recently Used (LRU) sector cache.
pub static mut SECTOR_CACHE: [CacheEntry; 16] = [CacheEntry {
    sector: 0,
    data: [0; 512],
    valid: false,
    last_used: 0,
}; 16];

/// Monotonic access clock for LRU eviction calculation.
pub static mut CACHE_CLOCK: u64 = 0;

/// Global cumulative cache telemetry counters.
pub static mut CACHE_HITS: u64 = 0;
pub static mut CACHE_MISSES: u64 = 0;
pub static mut CACHE_EVICTIONS: u64 = 0;

/// Retrieves global sector cache telemetry metrics: `(hits, misses, evictions, active_slots)`.
pub fn get_sector_cache_stats() -> (u64, u64, u64, usize) {
    unsafe {
        let active = SECTOR_CACHE.iter().filter(|e| e.valid).count();
        (CACHE_HITS, CACHE_MISSES, CACHE_EVICTIONS, active)
    }
}

/// Invalidates all entries currently held in the sector cache.
///
/// # Safety
///
/// Modifies the global mutable sector cache table.
pub unsafe fn clear_cache() {
    for entry in SECTOR_CACHE.iter_mut() {
        entry.valid = false;
    }
}
