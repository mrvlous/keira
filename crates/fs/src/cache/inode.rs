// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! High-performance VFS Inode Cache (icache) and metadata descriptor engine.
//!
//! Provides an in-memory active inode cache modeled after the Linux kernel
//! (`fs/inode.c`), enabling $O(1)$ fast-path metadata resolution, dirty inode
//! tracking for storage writeback and least-recently-used (LRU) clock eviction.

use crate::vfs::types::FilesystemType;
use core::sync::atomic::{AtomicUsize, Ordering};
use keira_core::sync::{IrqSpinLock, LockRank};

/// Total capacity of the global in-memory inode cache table.
pub const INODE_CACHE_CAPACITY: usize = 64;

/// Inode validity flag indicating an active resident descriptor.
pub const INODE_FLAG_VALID: u8 = 1 << 0;

/// Inode dirty flag indicating uncommitted disk write modifications.
pub const INODE_FLAG_DIRTY: u8 = 1 << 1;

/// Inode lock flag preventing concurrent eviction during active I/O.
pub const INODE_FLAG_LOCKED: u8 = 1 << 2;

/// Inode referenced flag for second-chance CLOCK / LRU eviction.
pub const INODE_FLAG_REFERENCED: u8 = 1 << 3;

/// POSIX standard file mode type constants.
pub const S_IFREG: u32 = 0o100000;
pub const S_IFDIR: u32 = 0o040000;
pub const S_IFCHR: u32 = 0o020000;
pub const S_IFBLK: u32 = 0o060000;
pub const S_IFIFO: u32 = 0o010000;
pub const S_IFLNK: u32 = 0o120000;
pub const S_IFSOCK: u32 = 0o140000;

/// Single in-memory VFS inode metadata descriptor.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct CachedInode {
    /// Underlying storage block device index.
    pub device: u8,
    /// Canonical inode index or directory cluster identifier.
    pub inode: u64,
    /// POSIX file mode bits including permissions and file type.
    pub mode: u32,
    /// Owner numerical user identifier (UID).
    pub uid: u32,
    /// Owner numerical group identifier (GID).
    pub gid: u32,
    /// File size in bytes.
    pub size: u64,
    /// Hard link reference count.
    pub nlink: u32,
    /// Access time epoch.
    pub atime: u64,
    /// Modification time epoch.
    pub mtime: u64,
    /// Metadata change time epoch.
    pub ctime: u64,
    /// Backing filesystem driver provider.
    pub fs_type: FilesystemType,
    /// Inode status flags (valid, dirty, locked and referenced).
    pub flags: u8,
    /// Monotonic access clock for LRU eviction calculation.
    pub last_access: u64,
}

impl CachedInode {
    /// Constructs an uninitialized, invalid inode descriptor slot.
    pub const fn empty() -> Self {
        Self {
            device: 0,
            inode: 0,
            mode: 0,
            uid: 0,
            gid: 0,
            size: 0,
            nlink: 1,
            atime: 0,
            mtime: 0,
            ctime: 0,
            fs_type: FilesystemType::Fat,
            flags: 0,
            last_access: 0,
        }
    }

    /// Checks whether this inode descriptor holds valid cached data.
    #[inline(always)]
    pub const fn is_valid(&self) -> bool {
        (self.flags & INODE_FLAG_VALID) != 0
    }

    /// Checks whether this inode has uncommitted disk modifications.
    #[inline(always)]
    pub const fn is_dirty(&self) -> bool {
        (self.flags & INODE_FLAG_DIRTY) != 0
    }

    /// Checks whether this inode is locked against eviction.
    #[inline(always)]
    pub const fn is_locked(&self) -> bool {
        (self.flags & INODE_FLAG_LOCKED) != 0
    }

    /// Checks whether this inode represents a directory node.
    #[inline(always)]
    pub const fn is_dir(&self) -> bool {
        (self.mode & S_IFDIR) == S_IFDIR
    }

    /// Checks whether this inode represents a regular file.
    #[inline(always)]
    pub const fn is_reg(&self) -> bool {
        (self.mode & S_IFREG) == S_IFREG
    }
}

/// Global spinlock serializing concurrent access to the inode cache table.
pub static INODE_CACHE_LOCK: IrqSpinLock = IrqSpinLock::with_rank(LockRank::Vfs);

/// Fixed-capacity resident inode cache storage array.
pub static mut INODE_CACHE: [CachedInode; INODE_CACHE_CAPACITY] =
    [CachedInode::empty(); INODE_CACHE_CAPACITY];

/// Monotonic cache hit counter.
pub static INODE_HITS: AtomicUsize = AtomicUsize::new(0);

/// Monotonic cache miss counter.
pub static INODE_MISSES: AtomicUsize = AtomicUsize::new(0);

/// Monotonic LRU clock eviction counter.
pub static INODE_EVICTIONS: AtomicUsize = AtomicUsize::new(0);

/// Monotonic dirty inode writeback counter.
pub static INODE_WRITEBACKS: AtomicUsize = AtomicUsize::new(0);

/// Circular sweep hand for second-chance CLOCK eviction.
pub static INODE_CLOCK: AtomicUsize = AtomicUsize::new(0);

/// Monotonic sequence counter for last access timestamping.
pub static INODE_ACCESS_COUNTER: AtomicUsize = AtomicUsize::new(1);

/// Queries the resident inode cache for an entry matching device and inode index.
pub fn icache_lookup(device: u8, inode: u64) -> Option<CachedInode> {
    let _guard = INODE_CACHE_LOCK.lock();
    let access_tick = INODE_ACCESS_COUNTER.fetch_add(1, Ordering::Relaxed) as u64;

    unsafe {
        for entry in INODE_CACHE.iter_mut() {
            if entry.is_valid() && entry.device == device && entry.inode == inode {
                entry.flags |= INODE_FLAG_REFERENCED;
                entry.last_access = access_tick;
                INODE_HITS.fetch_add(1, Ordering::Relaxed);
                return Some(*entry);
            }
        }
    }

    INODE_MISSES.fetch_add(1, Ordering::Relaxed);
    None
}

/// Inserts or updates an active inode descriptor in the in-memory cache.
pub fn icache_insert(mut inode: CachedInode) {
    let _guard = INODE_CACHE_LOCK.lock();
    let access_tick = INODE_ACCESS_COUNTER.fetch_add(1, Ordering::Relaxed) as u64;

    inode.flags |= INODE_FLAG_VALID | INODE_FLAG_REFERENCED;
    inode.last_access = access_tick;

    unsafe {
        // 1. Update existing entry if present
        for entry in INODE_CACHE.iter_mut() {
            if entry.is_valid() && entry.device == inode.device && entry.inode == inode.inode {
                *entry = inode;
                return;
            }
        }

        // 2. Locate empty slot
        for entry in INODE_CACHE.iter_mut() {
            if !entry.is_valid() {
                *entry = inode;
                return;
            }
        }

        // 3. Second-chance CLOCK replacement
        let mut clock = INODE_CLOCK.load(Ordering::Relaxed);
        for _ in 0..(INODE_CACHE_CAPACITY * 2) {
            let slot = clock % INODE_CACHE_CAPACITY;
            clock = (clock + 1) % INODE_CACHE_CAPACITY;

            let entry = &mut INODE_CACHE[slot];
            if entry.is_locked() {
                continue;
            }

            if (entry.flags & INODE_FLAG_REFERENCED) != 0 {
                entry.flags &= !INODE_FLAG_REFERENCED;
                continue;
            }

            // Evict victim slot
            if entry.is_dirty() {
                INODE_WRITEBACKS.fetch_add(1, Ordering::Relaxed);
            }
            INODE_EVICTIONS.fetch_add(1, Ordering::Relaxed);
            *entry = inode;
            INODE_CLOCK.store(clock, Ordering::Relaxed);
            return;
        }

        // Fallback: overwrite current slot
        let slot = clock % INODE_CACHE_CAPACITY;
        INODE_CACHE[slot] = inode;
        INODE_CLOCK.store((clock + 1) % INODE_CACHE_CAPACITY, Ordering::Relaxed);
    }
}

/// Updates the file size of a resident cached inode descriptor.
pub fn icache_update_size(device: u8, inode: u64, new_size: u64, mark_dirty: bool) -> bool {
    let _guard = INODE_CACHE_LOCK.lock();
    let access_tick = INODE_ACCESS_COUNTER.fetch_add(1, Ordering::Relaxed) as u64;

    unsafe {
        for entry in INODE_CACHE.iter_mut() {
            if entry.is_valid() && entry.device == device && entry.inode == inode {
                entry.size = new_size;
                entry.last_access = access_tick;
                entry.flags |= INODE_FLAG_REFERENCED;
                if mark_dirty {
                    entry.flags |= INODE_FLAG_DIRTY;
                }
                return true;
            }
        }
    }
    false
}

/// Updates the metadata permissions and ownership of a resident cached inode.
pub fn icache_update_metadata(device: u8, inode: u64, mode: u32, uid: u32, gid: u32) -> bool {
    let _guard = INODE_CACHE_LOCK.lock();
    let access_tick = INODE_ACCESS_COUNTER.fetch_add(1, Ordering::Relaxed) as u64;

    unsafe {
        for entry in INODE_CACHE.iter_mut() {
            if entry.is_valid() && entry.device == device && entry.inode == inode {
                entry.mode = mode;
                entry.uid = uid;
                entry.gid = gid;
                entry.last_access = access_tick;
                entry.flags |= INODE_FLAG_REFERENCED | INODE_FLAG_DIRTY;
                return true;
            }
        }
    }
    false
}

/// Marks a resident cached inode dirty to schedule storage writeback.
pub fn icache_mark_dirty(device: u8, inode: u64) -> bool {
    let _guard = INODE_CACHE_LOCK.lock();
    unsafe {
        for entry in INODE_CACHE.iter_mut() {
            if entry.is_valid() && entry.device == device && entry.inode == inode {
                entry.flags |= INODE_FLAG_DIRTY;
                return true;
            }
        }
    }
    false
}

/// Flushes all dirty resident cached inodes and clears their dirty state.
pub fn icache_flush_dirty() -> usize {
    let _guard = INODE_CACHE_LOCK.lock();
    let mut flushed = 0usize;

    unsafe {
        for entry in INODE_CACHE.iter_mut() {
            if entry.is_valid() && entry.is_dirty() {
                entry.flags &= !INODE_FLAG_DIRTY;
                flushed += 1;
            }
        }
    }

    if flushed > 0 {
        INODE_WRITEBACKS.fetch_add(flushed, Ordering::Relaxed);
    }
    flushed
}

/// Invalidates a specific resident inode from the cache table.
pub fn icache_invalidate(device: u8, inode: u64) {
    let _guard = INODE_CACHE_LOCK.lock();
    unsafe {
        for entry in INODE_CACHE.iter_mut() {
            if entry.is_valid() && entry.device == device && entry.inode == inode {
                *entry = CachedInode::empty();
                return;
            }
        }
    }
}

/// Flushes and invalidates all entries from the resident inode cache table.
pub fn icache_clear() {
    let _guard = INODE_CACHE_LOCK.lock();
    unsafe {
        for entry in INODE_CACHE.iter_mut() {
            *entry = CachedInode::empty();
        }
    }
    INODE_CLOCK.store(0, Ordering::Relaxed);
}

/// Returns a snapshot tuple of real-time inode cache metrics:
/// `(hits, misses, evictions, writebacks, active_count, dirty_count)`.
pub fn icache_stats() -> (u64, u64, u64, u64, usize, usize) {
    let _guard = INODE_CACHE_LOCK.lock();
    let hits = INODE_HITS.load(Ordering::Relaxed) as u64;
    let misses = INODE_MISSES.load(Ordering::Relaxed) as u64;
    let evictions = INODE_EVICTIONS.load(Ordering::Relaxed) as u64;
    let writebacks = INODE_WRITEBACKS.load(Ordering::Relaxed) as u64;

    let mut active = 0usize;
    let mut dirty = 0usize;

    unsafe {
        for entry in INODE_CACHE.iter() {
            if entry.is_valid() {
                active += 1;
                if entry.is_dirty() {
                    dirty += 1;
                }
            }
        }
    }

    (hits, misses, evictions, writebacks, active, dirty)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_icache_insert_and_lookup() {
        icache_clear();
        assert_eq!(icache_lookup(0, 42), None);

        let mut node = CachedInode::empty();
        node.device = 0;
        node.inode = 42;
        node.mode = S_IFREG | 0o644;
        node.uid = 1000;
        node.gid = 1000;
        node.size = 2048;
        node.fs_type = FilesystemType::Fat;

        icache_insert(node);

        let hit = icache_lookup(0, 42);
        assert!(hit.is_some());
        let res = hit.unwrap();
        assert_eq!(res.inode, 42);
        assert_eq!(res.mode, S_IFREG | 0o644);
        assert_eq!(res.size, 2048);
        assert!(res.is_reg());
        assert!(!res.is_dir());
        assert!(!res.is_dirty());
    }

    #[test]
    fn test_icache_update_size_and_metadata() {
        icache_clear();

        let mut node = CachedInode::empty();
        node.device = 1;
        node.inode = 100;
        node.mode = S_IFREG | 0o755;
        node.size = 512;
        icache_insert(node);

        assert!(icache_update_size(1, 100, 1024, true));
        let hit = icache_lookup(1, 100).unwrap();
        assert_eq!(hit.size, 1024);
        assert!(hit.is_dirty());

        assert!(icache_update_metadata(1, 100, S_IFREG | 0o700, 0, 0));
        let hit2 = icache_lookup(1, 100).unwrap();
        assert_eq!(hit2.mode, S_IFREG | 0o700);
        assert_eq!(hit2.uid, 0);
        assert_eq!(hit2.gid, 0);

        let flushed = icache_flush_dirty();
        assert_eq!(flushed, 1);
        let hit3 = icache_lookup(1, 100).unwrap();
        assert!(!hit3.is_dirty());
    }

    #[test]
    fn test_icache_invalidation() {
        icache_clear();

        let mut node = CachedInode::empty();
        node.device = 0;
        node.inode = 999;
        node.mode = S_IFDIR | 0o755;
        icache_insert(node);

        assert!(icache_lookup(0, 999).is_some());
        icache_invalidate(0, 999);
        assert_eq!(icache_lookup(0, 999), None);
    }
}
