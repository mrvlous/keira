<!-- SPDX-License-Identifier: GPL-2.0-only -->

# EXT4 File System Parser

This document details the read-only Linux EXT4 filesystem parser in Keira Kernel.

---

## Supported Features

* **Superblock Parsing**: Reads magic number `0xEF53`, block size (1KB, 2KB, 4KB), total blocks, and volume label.
* **Block Groups**: Traverses block group descriptor tables.
* **Extents Tree**: Traverses EXT4 extent headers, internal index nodes, and leaf extents to resolve contiguous file data blocks.

---

## Core API (`crates/fs/src/ext4/mod.rs`)

```rust
pub fn ext4_mount(sector_start: u32) -> Result<(), &'static str>;
pub fn ext4_read_file(path: &str, buf: &mut [u8]) -> Result<usize, &'static str>;
```
