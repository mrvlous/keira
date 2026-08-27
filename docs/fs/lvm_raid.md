<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Logical Volume Manager (LVM) & Software RAID

This document describes volume virtualization and software RAID array management in Keira Kernel.

---

## RAID Levels Supported

* **RAID 0 (Striping)**: Data is split across physical drives for high throughput.
* **RAID 1 (Mirroring)**: Full data duplication across mirror physical drives for fault tolerance.
* **RAID 5 (Distributed Parity)**: Block striping with XOR parity across 3 or more physical drives.

---

## Core API (`crates/fs/src/lvm/mod.rs`)

```rust
pub fn lvm_create_volume(name: &str, size_blocks: usize) -> Result<u32, &'static str>;
pub fn raid_create_array(level: u8, disks: &[u32]) -> Result<u32, &'static str>;
```
