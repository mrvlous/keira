// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! High-performance VFS Dentry Cache (dcache) and Inode lookup engine.
//!
//! Provides an in-memory directory entry cache inspired by the Linux kernel
//! (`fs/dcache.c`), enabling $O(1)$ fast-path path resolution, negative dentry
//! caching to eliminate redundant disk I/O and least-recently-used (LRU) eviction.

use crate::vfs::types::FilesystemType;

/// Maximum length of a cached canonical path string.
pub const DENTRY_PATH_MAX: usize = 96;

/// Total capacity of the global in-memory dentry cache table.
pub const DENTRY_CACHE_CAPACITY: usize = 64;

/// Dentry validity flag indicating an active resident entry.
pub const DENTRY_FLAG_VALID: u8 = 1 << 0;

/// Negative dentry flag indicating confirmed non-existence of path.
pub const DENTRY_FLAG_NEGATIVE: u8 = 1 << 1;

/// Directory flag indicating that the cached node is a directory.
pub const DENTRY_FLAG_DIRECTORY: u8 = 1 << 2;

/// Referenced flag for second-chance CLOCK / LRU eviction.
pub const DENTRY_FLAG_REFERENCED: u8 = 1 << 3;

/// Lookup outcome returned by `dcache_lookup`.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum DentryLookupResult {
    /// Positive dentry hit: target node exists with cached attributes.
    Positive {
        is_dir: bool,
        file_size: usize,
        fs_type: FilesystemType,
        inode: u64,
    },
    /// Negative dentry hit: target node is confirmed to not exist.
    Negative,
}

/// Single in-memory directory entry descriptor.
#[derive(Copy, Clone)]
pub struct Dentry {
    /// FNV-1a 32-bit hash of canonical path.
    pub path_hash: u32,
    /// Canonical absolute path buffer.
    pub path: [u8; DENTRY_PATH_MAX],
    /// Length of path in bytes.
    pub path_len: usize,
    /// Target filesystem provider.
    pub fs_type: FilesystemType,
    /// Inode index or cluster identifier.
    pub inode: u64,
    /// Cached file size in bytes (0 for directories).
    pub file_size: usize,
    /// Status flags (valid, negative, directory and referenced).
    pub flags: u8,
    /// Monotonic access tick for LRU calculation.
    pub last_access: u64,
}

impl Dentry {
    /// Creates an empty, invalid dentry slot.
    pub const fn empty() -> Self {
        Self {
            path_hash: 0,
            path: [0u8; DENTRY_PATH_MAX],
            path_len: 0,
            fs_type: FilesystemType::Fat,
            inode: 0,
            file_size: 0,
            flags: 0,
            last_access: 0,
        }
    }

    /// Checks whether this dentry slot holds valid cached data.
    #[inline(always)]
    pub const fn is_valid(&self) -> bool {
        (self.flags & DENTRY_FLAG_VALID) != 0
    }

    /// Checks whether this dentry represents a confirmed negative cache entry.
    #[inline(always)]
    pub const fn is_negative(&self) -> bool {
        (self.flags & DENTRY_FLAG_NEGATIVE) != 0
    }

    /// Checks whether this dentry represents a directory node.
    #[inline(always)]
    pub const fn is_dir(&self) -> bool {
        (self.flags & DENTRY_FLAG_DIRECTORY) != 0
    }

    /// Verifies if this dentry matches the queried canonical path string.
    #[inline]
    pub fn matches(&self, path: &str, hash: u32) -> bool {
        if !self.is_valid() || self.path_hash != hash || self.path_len != path.len() {
            return false;
        }
        &self.path[..self.path_len] == path.as_bytes()
    }
}

/// Global in-memory VFS dentry cache table.
pub static mut DENTRY_CACHE: [Dentry; DENTRY_CACHE_CAPACITY] =
    [Dentry::empty(); DENTRY_CACHE_CAPACITY];

/// Monotonic access clock for dentry LRU eviction calculation.
pub static mut DENTRY_CLOCK: u64 = 0;

/// Cumulative positive dentry cache hit counter.
pub static mut DENTRY_HITS: u64 = 0;

/// Cumulative dentry cache miss counter.
pub static mut DENTRY_MISSES: u64 = 0;

/// Cumulative negative dentry hit counter (saved disk lookups).
pub static mut DENTRY_NEGATIVE_HITS: u64 = 0;

/// Cumulative dentry eviction counter.
pub static mut DENTRY_EVICTIONS: u64 = 0;

/// Computes an FNV-1a 32-bit hash for a canonical path string.
#[inline]
pub fn dentry_hash(path: &str) -> u32 {
    let mut hash: u32 = 0x811c9dc5;
    for &b in path.as_bytes() {
        hash ^= b as u32;
        hash = hash.wrapping_mul(0x01000193);
    }
    hash
}

/// Normalizes a path string by stripping trailing slashes (except root "/").
fn normalize_dentry_path(path: &str) -> &str {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return "/";
    }
    if trimmed.len() > 1 && trimmed.ends_with('/') {
        &trimmed[..trimmed.len() - 1]
    } else {
        trimmed
    }
}

/// Probes the in-memory dentry cache for a canonical path.
///
/// Returns `Some(DentryLookupResult)` on cache hit, or `None` on cache miss.
pub fn dcache_lookup(path: &str) -> Option<DentryLookupResult> {
    let norm = normalize_dentry_path(path);
    if norm.len() > DENTRY_PATH_MAX {
        return None;
    }
    let hash = dentry_hash(norm);

    unsafe {
        for entry in DENTRY_CACHE.iter_mut() {
            if entry.matches(norm, hash) {
                DENTRY_CLOCK = DENTRY_CLOCK.wrapping_add(1);
                entry.last_access = DENTRY_CLOCK;
                entry.flags |= DENTRY_FLAG_REFERENCED;

                if entry.is_negative() {
                    DENTRY_NEGATIVE_HITS = DENTRY_NEGATIVE_HITS.wrapping_add(1);
                    return Some(DentryLookupResult::Negative);
                }

                DENTRY_HITS = DENTRY_HITS.wrapping_add(1);
                return Some(DentryLookupResult::Positive {
                    is_dir: entry.is_dir(),
                    file_size: entry.file_size,
                    fs_type: entry.fs_type,
                    inode: entry.inode,
                });
            }
        }

        DENTRY_MISSES = DENTRY_MISSES.wrapping_add(1);
        None
    }
}

/// Inserts or updates a positive dentry in the cache.
///
/// If the cache table is full, an LRU entry without active reference is evicted.
pub fn dcache_add(path: &str, is_dir: bool, file_size: usize, fs_type: FilesystemType, inode: u64) {
    let norm = normalize_dentry_path(path);
    if norm.len() > DENTRY_PATH_MAX {
        return;
    }
    let hash = dentry_hash(norm);

    unsafe {
        DENTRY_CLOCK = DENTRY_CLOCK.wrapping_add(1);

        // 1. Check if entry already exists to update it in place
        for entry in DENTRY_CACHE.iter_mut() {
            if entry.matches(norm, hash) {
                entry.fs_type = fs_type;
                entry.inode = inode;
                entry.file_size = file_size;
                let mut flags = DENTRY_FLAG_VALID | DENTRY_FLAG_REFERENCED;
                if is_dir {
                    flags |= DENTRY_FLAG_DIRECTORY;
                }
                entry.flags = flags;
                entry.last_access = DENTRY_CLOCK;
                return;
            }
        }

        // 2. Find an empty slot
        for entry in DENTRY_CACHE.iter_mut() {
            if !entry.is_valid() {
                fill_dentry_slot(entry, norm, hash, is_dir, false, file_size, fs_type, inode);
                return;
            }
        }

        // 3. Table full: find LRU slot to evict
        let mut victim_idx = 0;
        let mut oldest_access = u64::MAX;

        for (idx, entry) in DENTRY_CACHE.iter().enumerate() {
            if entry.last_access < oldest_access {
                oldest_access = entry.last_access;
                victim_idx = idx;
            }
        }

        DENTRY_EVICTIONS = DENTRY_EVICTIONS.wrapping_add(1);
        let entry = &mut DENTRY_CACHE[victim_idx];
        fill_dentry_slot(entry, norm, hash, is_dir, false, file_size, fs_type, inode);
    }
}

/// Inserts a negative dentry into the cache (recording that path does not exist).
pub fn dcache_add_negative(path: &str) {
    let norm = normalize_dentry_path(path);
    if norm.len() > DENTRY_PATH_MAX {
        return;
    }
    let hash = dentry_hash(norm);

    unsafe {
        DENTRY_CLOCK = DENTRY_CLOCK.wrapping_add(1);

        for entry in DENTRY_CACHE.iter_mut() {
            if entry.matches(norm, hash) {
                entry.flags = DENTRY_FLAG_VALID | DENTRY_FLAG_NEGATIVE | DENTRY_FLAG_REFERENCED;
                entry.last_access = DENTRY_CLOCK;
                return;
            }
        }

        for entry in DENTRY_CACHE.iter_mut() {
            if !entry.is_valid() {
                fill_dentry_slot(entry, norm, hash, false, true, 0, FilesystemType::Fat, 0);
                return;
            }
        }

        let mut victim_idx = 0;
        let mut oldest_access = u64::MAX;

        for (idx, entry) in DENTRY_CACHE.iter().enumerate() {
            if entry.last_access < oldest_access {
                oldest_access = entry.last_access;
                victim_idx = idx;
            }
        }

        DENTRY_EVICTIONS = DENTRY_EVICTIONS.wrapping_add(1);
        let entry = &mut DENTRY_CACHE[victim_idx];
        fill_dentry_slot(entry, norm, hash, false, true, 0, FilesystemType::Fat, 0);
    }
}

#[inline]
fn fill_dentry_slot(
    slot: &mut Dentry,
    path: &str,
    hash: u32,
    is_dir: bool,
    is_negative: bool,
    file_size: usize,
    fs_type: FilesystemType,
    inode: u64,
) {
    slot.path_hash = hash;
    slot.path_len = path.len();
    slot.path[..path.len()].copy_from_slice(path.as_bytes());
    slot.fs_type = fs_type;
    slot.inode = inode;
    slot.file_size = file_size;
    unsafe {
        slot.last_access = DENTRY_CLOCK;
    }

    let mut flags = DENTRY_FLAG_VALID | DENTRY_FLAG_REFERENCED;
    if is_negative {
        flags |= DENTRY_FLAG_NEGATIVE;
    }
    if is_dir {
        flags |= DENTRY_FLAG_DIRECTORY;
    }
    slot.flags = flags;
}

/// Invalidates any cached dentry matching `path` or having `path` as directory prefix.
pub fn dcache_invalidate(path: &str) {
    let norm = normalize_dentry_path(path);
    let hash = dentry_hash(norm);

    unsafe {
        for entry in DENTRY_CACHE.iter_mut() {
            if !entry.is_valid() {
                continue;
            }
            if entry.matches(norm, hash) {
                entry.flags = 0;
                continue;
            }
            // Check if entry is a child under this directory
            if norm != "/" && entry.path_len > norm.len() {
                if let Ok(entry_path) = core::str::from_utf8(&entry.path[..entry.path_len]) {
                    if entry_path.starts_with(norm)
                        && entry_path.as_bytes().get(norm.len()) == Some(&b'/')
                    {
                        entry.flags = 0;
                    }
                }
            }
        }
    }
}

/// Clears and invalidates all entries in the global dentry cache table.
///
/// # Safety
///
/// Modifies the global mutable dentry cache table.
pub unsafe fn dcache_clear() {
    for entry in DENTRY_CACHE.iter_mut() {
        entry.flags = 0;
        entry.path_len = 0;
    }
}

/// Retrieves real-time dentry cache telemetry metrics:
/// `(hits, misses, negative_hits, evictions, active_positive, active_negative)`.
pub fn dcache_stats() -> (u64, u64, u64, u64, usize, usize) {
    unsafe {
        let mut active_pos = 0;
        let mut active_neg = 0;

        for entry in DENTRY_CACHE.iter() {
            if entry.is_valid() {
                if entry.is_negative() {
                    active_neg += 1;
                } else {
                    active_pos += 1;
                }
            }
        }

        (
            DENTRY_HITS,
            DENTRY_MISSES,
            DENTRY_NEGATIVE_HITS,
            DENTRY_EVICTIONS,
            active_pos,
            active_neg,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dcache_positive_and_negative_lookup() {
        unsafe {
            dcache_clear();
        }

        assert_eq!(dcache_lookup("/bin/sh.elf"), None);

        dcache_add("/bin/sh.elf", false, 12345, FilesystemType::Fat, 42);

        let res = dcache_lookup("/bin/sh.elf");
        assert_eq!(
            res,
            Some(DentryLookupResult::Positive {
                is_dir: false,
                file_size: 12345,
                fs_type: FilesystemType::Fat,
                inode: 42,
            })
        );

        dcache_add_negative("/nonexistent.txt");
        assert_eq!(
            dcache_lookup("/nonexistent.txt"),
            Some(DentryLookupResult::Negative)
        );

        dcache_invalidate("/bin/sh.elf");
        assert_eq!(dcache_lookup("/bin/sh.elf"), None);
    }

    #[test]
    fn test_dcache_prefix_invalidation() {
        unsafe {
            dcache_clear();
        }

        dcache_add("/tmp/a.txt", false, 10, FilesystemType::Tmpfs, 1);
        dcache_add("/tmp/b.txt", false, 20, FilesystemType::Tmpfs, 2);
        dcache_add("/bin/cat.elf", false, 50, FilesystemType::Fat, 3);

        assert!(dcache_lookup("/tmp/a.txt").is_some());
        assert!(dcache_lookup("/tmp/b.txt").is_some());
        assert!(dcache_lookup("/bin/cat.elf").is_some());

        dcache_invalidate("/tmp");

        assert_eq!(dcache_lookup("/tmp/a.txt"), None);
        assert_eq!(dcache_lookup("/tmp/b.txt"), None);
        assert!(dcache_lookup("/bin/cat.elf").is_some());
    }
}
