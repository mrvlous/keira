// SPDX-License-Identifier: GPL-2.0-only

//! FAT sector caching engine and block device I/O abstractions.

pub mod cache;
pub mod io;

pub use self::cache::{clear_cache, get_sector_cache_stats, CacheEntry, CACHE_CLOCK, SECTOR_CACHE};
pub use self::io::{flush_dirty_sectors, read_sector, write_sector};
