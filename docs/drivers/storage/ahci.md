<!-- SPDX-License-Identifier: GPL-2.0-only -->

# AHCI SATA Storage Controller Driver

This document specifies the Advanced Host Controller Interface (AHCI) driver in Keira Kernel.

---

## Technical Specifications

* **Command Queuing**: 32-slot command header list with Command Table PRDT (Physical Region Descriptor Tables).
* **Frame Information Structure (FIS)**: Register Host-to-Device (H2D) FIS generation.
* **DMA Operations**: High-throughput direct memory access transfers without CPU polling.

---

## Core API (`crates/io/src/storage/ahci.rs`)

```rust
pub unsafe fn ahci_init(bar5: usize) -> Result<(), &'static str>;
pub unsafe fn ahci_read_sector(port: usize, lba: u64, buf: &mut [u8; 512]) -> Result<(), &'static str>;
pub unsafe fn ahci_write_sector(port: usize, lba: u64, buf: &[u8; 512]) -> Result<(), &'static str>;
```
