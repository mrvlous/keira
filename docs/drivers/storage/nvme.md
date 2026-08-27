<!-- SPDX-License-Identifier: GPL-2.0-only -->

# NVMe PCIe Storage Controller Driver

This document details the Non-Volatile Memory Express (NVMe) solid-state storage driver in Keira Kernel.

---

## Technical Specifications

* **Queue Pairs**: Admin Submission Queue (ASQ) / Completion Queue (ACQ) and I/O Submission Queue (IOSQ) / Completion Queue (IOCQ).
* **Doorbell Registers**: Memory-mapped doorbell register updates triggering PCIe DMA transfers.
* **Block Size**: 512 bytes / 4096 bytes per logical block address.

---

## Core API (`crates/io/src/storage/nvme.rs`)

```rust
pub unsafe fn nvme_init(bar0: usize) -> Result<(), &'static str>;
pub unsafe fn nvme_read_blocks(lba: u64, count: u16, buf: &mut [u8]) -> Result<(), &'static str>;
pub unsafe fn nvme_write_blocks(lba: u64, count: u16, buf: &[u8]) -> Result<(), &'static str>;
```
