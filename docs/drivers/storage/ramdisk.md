<!-- SPDX-License-Identifier: GPL-2.0-only -->

# RAM Disk Block Storage Driver

This document specifies the in-memory volatile block storage device driver in Keira Kernel.

---

## Technical Specifications

* **Memory Backing**: Contiguous physical RAM allocated at boot time.
* **Access Speed**: Zero-latency memory copy (`memcpy`) throughput without bus overhead.
* **Block Size**: 512 bytes per sector.

---

## Core API (`crates/io/src/storage/ramdisk.rs`)

```rust
pub fn ramdisk_init(size_mb: usize) -> Result<(), &'static str>;
pub fn ramdisk_read_sector(sector: u32, buf: &mut [u8; 512]) -> Result<(), &'static str>;
pub fn ramdisk_write_sector(sector: u32, buf: &[u8; 512]) -> Result<(), &'static str>;
```
