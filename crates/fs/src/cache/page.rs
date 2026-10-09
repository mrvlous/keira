// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Unified 4 KiB page cache engine, dirty writeback tracking and dynamic buffer management.
//!
//! Provides a resident 4096-byte page cache bridging the Virtual Filesystem (VFS)
//! with physical memory frame allocation (PMM) and demand-paged address spaces.

/// Granular page size matching CPU architectural frame size (4 KiB).
pub const PAGE_SIZE: usize = 4096;

/// Total capacity of the global unified page cache pool (32 pages = 128 KiB).
pub const PAGE_CACHE_CAPACITY: usize = 32;

/// Page validity flag indicating resident cached data.
pub const PAGE_FLAG_VALID: u8 = 1 << 0;

/// Page dirty flag indicating uncommitted disk write modifications.
pub const PAGE_FLAG_DIRTY: u8 = 1 << 1;

/// Page lock flag preventing concurrent eviction during active I/O.
pub const PAGE_FLAG_LOCKED: u8 = 1 << 2;

/// Page referenced flag for CLOCK / second-chance eviction tracking.
pub const PAGE_FLAG_REFERENCED: u8 = 1 << 3;

/// Single entry in the unified page cache pool.
#[derive(Copy, Clone)]
pub struct PageCacheEntry {
    /// Underlying storage block device index.
    pub device: u8,
    /// Inode identifier or directory cluster index.
    pub inode: u32,
    /// Offset within the file counted in 4096-byte page units.
    pub page_index: u32,
    /// Status flags (valid, dirty, locked and referenced).
    pub flags: u8,
    /// Monotonic access clock for LRU eviction calculation.
    pub last_access: u64,
    /// Number of valid data bytes present in this page (up to 4096).
    pub valid_len: usize,
    /// Resident 4096-byte raw data buffer.
    pub data: [u8; PAGE_SIZE],
}

impl PageCacheEntry {
    /// Creates an uninitialized, invalid page cache entry.
    pub const fn empty() -> Self {
        Self {
            device: 0,
            inode: 0,
            page_index: 0,
            flags: 0,
            last_access: 0,
            valid_len: 0,
            data: [0u8; PAGE_SIZE],
        }
    }

    /// Checks whether this page contains valid resident data.
    pub const fn is_valid(&self) -> bool {
        (self.flags & PAGE_FLAG_VALID) != 0
    }

    /// Checks whether this page has uncommitted modifications.
    pub const fn is_dirty(&self) -> bool {
        (self.flags & PAGE_FLAG_DIRTY) != 0
    }

    /// Checks whether this page is locked against eviction.
    pub const fn is_locked(&self) -> bool {
        (self.flags & PAGE_FLAG_LOCKED) != 0
    }
}

/// Global unified page cache storage pool.
pub static mut PAGE_CACHE: [PageCacheEntry; PAGE_CACHE_CAPACITY] =
    [PageCacheEntry::empty(); PAGE_CACHE_CAPACITY];

/// Monotonic access clock for LRU calculation.
pub static mut PAGE_CLOCK: u64 = 0;

/// Cumulative cache hit counter.
pub static mut PAGE_HITS: u64 = 0;

/// Cumulative cache miss counter.
pub static mut PAGE_MISSES: u64 = 0;

/// Cumulative LRU eviction counter.
pub static mut PAGE_EVICTIONS: u64 = 0;

/// Cumulative dirty writeback flush counter.
pub static mut PAGE_WRITEBACKS: u64 = 0;

/// Retrieves telemetry statistics: `(hits, misses, evictions, writebacks, active_pages)`.
pub fn get_page_cache_stats() -> (u64, u64, u64, u64, usize) {
    unsafe {
        let active = PAGE_CACHE.iter().filter(|e| e.is_valid()).count();
        (
            PAGE_HITS,
            PAGE_MISSES,
            PAGE_EVICTIONS,
            PAGE_WRITEBACKS,
            active,
        )
    }
}

/// Invalidates all entries currently held in the unified page cache.
///
/// # Safety
///
/// Modifies the global mutable page cache table.
pub unsafe fn clear_page_cache() {
    for entry in PAGE_CACHE.iter_mut() {
        entry.flags = 0;
        entry.valid_len = 0;
    }
}

/// Invalidates all cached pages associated with a specific inode.
///
/// # Safety
///
/// Scans and modifies matching entries in the global mutable page cache.
pub unsafe fn invalidate_inode(device: u8, inode: u32) {
    for entry in PAGE_CACHE.iter_mut() {
        if entry.is_valid() && entry.device == device && entry.inode == inode {
            entry.flags = 0;
            entry.valid_len = 0;
        }
    }
}

/// Finds the slot index of a cached page matching device, inode and page offset.
///
/// # Safety
///
/// Reads the global mutable page cache table and advances the access clock.
pub unsafe fn find_page_index(device: u8, inode: u32, page_index: u32) -> Option<usize> {
    PAGE_CLOCK = PAGE_CLOCK.wrapping_add(1);
    for (idx, entry) in PAGE_CACHE.iter_mut().enumerate() {
        if entry.is_valid()
            && entry.device == device
            && entry.inode == inode
            && entry.page_index == page_index
        {
            entry.last_access = PAGE_CLOCK;
            entry.flags |= PAGE_FLAG_REFERENCED;
            PAGE_HITS = PAGE_HITS.wrapping_add(1);
            return Some(idx);
        }
    }
    PAGE_MISSES = PAGE_MISSES.wrapping_add(1);
    None
}

/// Reads data from a cached page into a caller-provided destination slice.
///
/// Returns the number of bytes read if the page exists in cache.
///
/// # Safety
///
/// Accesses the global page cache table and copies resident data bytes.
pub unsafe fn read_page(
    device: u8,
    inode: u32,
    page_index: u32,
    offset_in_page: usize,
    out: &mut [u8],
) -> Option<usize> {
    if offset_in_page >= PAGE_SIZE {
        return None;
    }

    let slot = find_page_index(device, inode, page_index)?;
    let entry = &PAGE_CACHE[slot];

    if offset_in_page >= entry.valid_len {
        return Some(0);
    }

    let available = entry.valid_len - offset_in_page;
    let to_copy = out.len().min(available);
    out[..to_copy].copy_from_slice(&entry.data[offset_in_page..offset_in_page + to_copy]);
    Some(to_copy)
}

/// Allocates a slot in the page cache, evicting an existing page via LRU if necessary.
unsafe fn allocate_slot() -> usize {
    // 1. Look for an empty slot
    for (idx, entry) in PAGE_CACHE.iter().enumerate() {
        if !entry.is_valid() {
            return idx;
        }
    }

    // 2. Evict least recently used unlocked entry
    let mut oldest_slot = 0;
    let mut min_access = u64::MAX;

    for (idx, entry) in PAGE_CACHE.iter().enumerate() {
        if !entry.is_locked() && entry.last_access < min_access {
            min_access = entry.last_access;
            oldest_slot = idx;
        }
    }

    let evict_entry = &mut PAGE_CACHE[oldest_slot];
    if evict_entry.is_dirty() {
        PAGE_WRITEBACKS = PAGE_WRITEBACKS.wrapping_add(1);
    }
    PAGE_EVICTIONS = PAGE_EVICTIONS.wrapping_add(1);
    evict_entry.flags = 0;
    evict_entry.valid_len = 0;

    oldest_slot
}

/// Inserts a new 4 KiB page into the cache.
///
/// # Safety
///
/// Allocates a cache slot and mutates global page cache state.
pub unsafe fn insert_page(
    device: u8,
    inode: u32,
    page_index: u32,
    src: &[u8],
    dirty: bool,
) -> usize {
    PAGE_CLOCK = PAGE_CLOCK.wrapping_add(1);

    // If already exists, overwrite
    let mut found_slot = None;
    for (idx, entry) in PAGE_CACHE.iter().enumerate() {
        if entry.is_valid()
            && entry.device == device
            && entry.inode == inode
            && entry.page_index == page_index
        {
            found_slot = Some(idx);
            break;
        }
    }
    let slot = match found_slot {
        Some(s) => s,
        None => allocate_slot(),
    };

    let entry = &mut PAGE_CACHE[slot];
    entry.device = device;
    entry.inode = inode;
    entry.page_index = page_index;
    entry.last_access = PAGE_CLOCK;

    let copy_len = src.len().min(PAGE_SIZE);
    entry.data[..copy_len].copy_from_slice(&src[..copy_len]);
    if copy_len < PAGE_SIZE {
        entry.data[copy_len..].fill(0);
    }
    entry.valid_len = copy_len;

    let mut flags = PAGE_FLAG_VALID | PAGE_FLAG_REFERENCED;
    if dirty {
        flags |= PAGE_FLAG_DIRTY;
    }
    entry.flags = flags;

    slot
}

/// Writes data into a cached page, marking it dirty.
///
/// # Safety
///
/// Mutates page cache content and updates dirty tracking flags.
pub unsafe fn write_page(
    device: u8,
    inode: u32,
    page_index: u32,
    offset_in_page: usize,
    src: &[u8],
) -> Result<usize, &'static str> {
    if offset_in_page >= PAGE_SIZE {
        return Err("Page cache write offset exceeds 4096-byte page boundary");
    }

    PAGE_CLOCK = PAGE_CLOCK.wrapping_add(1);

    let slot = match find_page_index(device, inode, page_index) {
        Some(s) => s,
        None => {
            // Allocate empty page
            let s = allocate_slot();
            let entry = &mut PAGE_CACHE[s];
            entry.device = device;
            entry.inode = inode;
            entry.page_index = page_index;
            entry.valid_len = 0;
            entry.data.fill(0);
            entry.flags = PAGE_FLAG_VALID;
            s
        }
    };

    let entry = &mut PAGE_CACHE[slot];
    let available = PAGE_SIZE - offset_in_page;
    let to_write = src.len().min(available);

    entry.data[offset_in_page..offset_in_page + to_write].copy_from_slice(&src[..to_write]);
    let new_len = offset_in_page + to_write;
    if new_len > entry.valid_len {
        entry.valid_len = new_len;
    }

    entry.flags |= PAGE_FLAG_DIRTY | PAGE_FLAG_REFERENCED;
    entry.last_access = PAGE_CLOCK;

    Ok(to_write)
}

/// Flushes all dirty pages to permanent storage using a supplied callback closure.
///
/// # Safety
///
/// Iterates over global page cache entries and clears dirty flags upon successful flush.
pub unsafe fn flush_dirty_pages<F>(mut flush_fn: F) -> usize
where
    F: FnMut(u8, u32, u32, &[u8]) -> Result<(), &'static str>,
{
    let mut flushed_count = 0;

    for entry in PAGE_CACHE.iter_mut() {
        if entry.is_valid() && entry.is_dirty() {
            entry.flags |= PAGE_FLAG_LOCKED;
            let res = flush_fn(
                entry.device,
                entry.inode,
                entry.page_index,
                &entry.data[..entry.valid_len],
            );
            entry.flags &= !PAGE_FLAG_LOCKED;

            if res.is_ok() {
                entry.flags &= !PAGE_FLAG_DIRTY;
                flushed_count += 1;
                PAGE_WRITEBACKS = PAGE_WRITEBACKS.wrapping_add(1);
            }
        }
    }

    flushed_count
}
