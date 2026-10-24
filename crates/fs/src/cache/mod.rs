// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Unified page cache, dentry cache and inode cache subsystem.

pub mod dentry;
pub mod inode;
pub mod page;

pub use self::dentry::{
    dcache_add, dcache_add_negative, dcache_clear, dcache_invalidate, dcache_lookup, dcache_stats,
    dentry_hash, Dentry, DentryLookupResult, DENTRY_CACHE, DENTRY_CACHE_CAPACITY, DENTRY_CLOCK,
    DENTRY_EVICTIONS, DENTRY_FLAG_DIRECTORY, DENTRY_FLAG_NEGATIVE, DENTRY_FLAG_REFERENCED,
    DENTRY_FLAG_VALID, DENTRY_HITS, DENTRY_MISSES, DENTRY_NEGATIVE_HITS, DENTRY_PATH_MAX,
};
pub use self::inode::{
    icache_clear, icache_flush_dirty, icache_insert, icache_invalidate, icache_lookup,
    icache_mark_dirty, icache_stats, icache_update_metadata, icache_update_size, CachedInode,
    INODE_CACHE, INODE_CACHE_CAPACITY, INODE_CACHE_LOCK, INODE_CLOCK, INODE_EVICTIONS,
    INODE_FLAG_DIRTY, INODE_FLAG_LOCKED, INODE_FLAG_REFERENCED, INODE_FLAG_VALID, INODE_HITS,
    INODE_MISSES, INODE_WRITEBACKS, S_IFBLK, S_IFCHR, S_IFDIR, S_IFIFO, S_IFLNK, S_IFREG, S_IFSOCK,
};
pub use self::page::{
    clear_page_cache, find_page_index, flush_dirty_pages, get_page_cache_stats, insert_page,
    invalidate_inode, read_page, write_page, PageCacheEntry, PAGE_CACHE, PAGE_CACHE_CAPACITY,
    PAGE_CLOCK, PAGE_EVICTIONS, PAGE_FLAG_DIRTY, PAGE_FLAG_LOCKED, PAGE_FLAG_REFERENCED,
    PAGE_FLAG_VALID, PAGE_HITS, PAGE_MISSES, PAGE_SIZE, PAGE_WRITEBACKS,
};
