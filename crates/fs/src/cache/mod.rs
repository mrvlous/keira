// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Unified buffer cache and page management subsystem.

pub mod page;

pub use self::page::{
    clear_page_cache, find_page_index, flush_dirty_pages, get_page_cache_stats, insert_page,
    invalidate_inode, read_page, write_page, PageCacheEntry, PAGE_CACHE, PAGE_CACHE_CAPACITY,
    PAGE_CLOCK, PAGE_EVICTIONS, PAGE_FLAG_DIRTY, PAGE_FLAG_LOCKED, PAGE_FLAG_REFERENCED,
    PAGE_FLAG_VALID, PAGE_HITS, PAGE_MISSES, PAGE_SIZE, PAGE_WRITEBACKS,
};
