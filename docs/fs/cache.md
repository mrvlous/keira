<!-- SPDX-License-Identifier: GPL-2.0-only -->

# 16-Slot LRU Write-Through Sector Cache

This document details the in-memory sector cache engine accelerating block storage access in Keira Kernel.

---

## Technical Specifications

* **Cache Capacity**: 16 slots $\times$ 512 bytes = 8192 bytes.
* **Eviction Policy**: Least Recently Used (LRU) tracking with a monotonic cache clock counter.
* **Write Policy**: Write-through caching (updates cache slot and writes immediately to physical storage).
* **Synchronization**: Guarded by `keira_core::sync::SpinMutex`.

---

## Core API (`crates/fs/src/fat/table.rs`)

```rust
pub fn clear_cache();
pub fn read_sector(sector: u32, buffer: &mut [u8; 512]) -> Result<(), &'static str>;
pub fn write_sector(sector: u32, buffer: &[u8; 512]) -> Result<(), &'static str>;
pub fn flush_dirty_sectors() -> Result<usize, &'static str>;
```
