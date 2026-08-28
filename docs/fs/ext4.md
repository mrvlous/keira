<!-- SPDX-License-Identifier: GPL-2.0-only -->

# EXT4 File System Read-Only Driver

This document specifies the fourth extended filesystem (EXT4) parser, inode extent tree navigation, and block group descriptors in Keira Kernel.

---

## EXT4 Extent Tree Traversal

```mermaid
graph TD
    Superblock["Superblock (1024 bytes @ Offset 1024)"] --> BlockGroup["Block Group Descriptor Table"]
    BlockGroup --> InodeTable["Inode Table Allocation"]
    InodeTable --> ExtentHeader["Extent Tree Header (eh_magic = 0xF30A)"]
    ExtentHeader --> ExtentNode{"eh_depth == 0?"}
    ExtentNode -->|Leaf (eh_depth=0)| ExtentLeaf["Read Data Blocks Directly (ee_start_lo/hi)"]
    ExtentNode -->|Index (eh_depth>0)| ExtentIndex["Traverse Sub-Tree Index Nodes"]
```

---

## Technical Specifications

| Parameter | Specification | Description |
| :--- | :--- | :--- |
| **Superblock Magic** | `0xEF53` | Standard EXT2/EXT3/EXT4 filesystem signature |
| **Block Sizes** | 1024, 2048, 4096 bytes | Computed as $1024 \times 2^{\text{s\_log\_block\_size}}$ |
| **Extent Magic** | `0xF30A` | Extent tree header signature |
| **Directory Format** | Linear & HTree Indexed | Hash-tree indexed directory entries |

---

## Core API (`crates/fs/src/ext4/mod.rs`)

```rust
/// Mount and initialize an EXT4 filesystem on a block device.
pub unsafe fn mount(device_id: usize) -> Result<(), &'static str>;

/// Read file contents from an EXT4 inode using extent tree navigation.
pub unsafe fn read_inode(inode_nr: u32, offset: u64, buf: &mut [u8]) -> Result<usize, &'static str>;
```
