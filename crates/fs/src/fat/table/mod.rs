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

//! FAT sector caching engine and block device I/O abstractions.

pub mod cache;
pub mod io;

pub use self::cache::{clear_cache, get_sector_cache_stats, CacheEntry, CACHE_CLOCK, SECTOR_CACHE};
pub use self::io::{flush_dirty_sectors, read_sector, write_sector};
