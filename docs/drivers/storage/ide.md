<!-- SPDX-License-Identifier: GPL-2.0-only -->

# IDE / Parallel ATA Block Driver

This document details the legacy IDE ATA PIO and Bus Master DMA storage driver in Keira Kernel.

---

## Hardware I/O Ports

* **Primary Bus**: Data (`0x1F0`), Error/Features (`0x1F1`), Sector Count (`0x1F2`), LBA Low (`0x1F3`), LBA Mid (`0x1F4`), LBA High (`0x1F5`), Drive/Head (`0x1F6`), Status/Command (`0x1F7`).
* **Control Port**: Device Control / Alternate Status (`0x3F6`).

---

## Read/Write Operations (`crates/io/src/storage/ide.rs`)

```rust
pub unsafe fn ide_read_sector(lba: u32, buf: &mut [u8; 512]) -> Result<(), &'static str>;
pub unsafe fn ide_write_sector(lba: u32, buf: &[u8; 512]) -> Result<(), &'static str>;
```
