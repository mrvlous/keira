<!-- SPDX-License-Identifier: GPL-2.0-only -->

# FAT File System Driver (FAT12 / FAT16 / FAT32)

This document specifies the FAT filesystem implementations in Keira Kernel.

---

## Technical Specifications

* **Supported Types**: FAT12 (Floppy), FAT16 (Primary Hard Disk Partition), FAT32 (Large Storage).
* **Cluster Allocation**: File Allocation Table (FAT) cluster chaining with end-of-chain marker (`0xFFFF` / `0x0FFFFFF8`).
* **Short 8.3 Filenames**: Standard 8-character filename + 3-character extension formatting.

---

## Core API (`crates/fs/src/fat/mod.rs`)

```rust
pub fn init() -> Result<(), &'static str>;
pub fn read_file(path: &str, buffer: &mut [u8]) -> Result<usize, &'static str>;
pub fn write_file(path: &str, data: &[u8]) -> Result<(), &'static str>;
pub fn create_file(path: &str) -> Result<(), &'static str>;
pub fn delete_file(path: &str) -> Result<(), &'static str>;
pub fn list_directory(path: &str) -> Result<(), &'static str>;
```
