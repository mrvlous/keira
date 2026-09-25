<!-- SPDX-License-Identifier: GPL-2.0-only -->

# FAT Directory Parsing, File Allocation & Sector Cache

Creates, reads, writes, and removes 8.3 and Long File Name (LFN) directory entries.

---

## Sector LRU Cache & I/O Buffering

The FAT table and sector layer (`crates/fs/src/fat/table/cache.rs`) incorporates an LRU sector cache to accelerate sector I/O across storage blocks:
* **Cache Architecture**: Fixed-capacity 16-slot sector cache buffering active 512-byte sectors in memory.
* **Dirty Sector Writeback**: Dirty cache lines are tracked and flushed back to underlying block devices during sync or eviction.
* **LRU Replacement Policy**: Least-recently-used sectors are dynamically identified by tick order and replaced when cache capacity is reached.
* **Telemetry Counters**: Tracks cache hits, misses, and evictions atomically (`get_sector_cache_stats()`).
* **Disk Reporting**: Reported via `disk` shell command, computing hit ratio percentages and slot utilization.
