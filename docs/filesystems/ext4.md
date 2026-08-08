<!--
SPDX-License-Identifier: GPL-2.0-only

Keira Kernel - Operating System Kernel
Copyright (C) 2026 Moh. Ananda Firmansyah Putra
-->

# EXT4 / EXT2 Linux Filesystem Kernel Driver Subsystem

This document details the architecture, data structures, superblock parsing, inode table reading, and extent tree mapping of the native EXT4 / EXT2 filesystem driver in Keira Kernel.

---

## 1. Subsystem Overview

Keira Kernel provides native read/write support for EXT4 and EXT2 Linux disk partitions ([ext4.rs](../../kernel/src/fs/ext4.rs)). Unlike legacy FAT16 filesystems, EXT4 provides true POSIX storage features including 32-bit inode numbers, block group descriptors, extended file attributes, and 48-bit physical block extent mapping.

```
+-------------------------------------------------------------------------+
|                              VFS Layer                                  |
+-------------------------------------------------------------------------+
                                    |
                                    v
+-------------------------------------------------------------------------+
|                  EXT4 / EXT2 Filesystem Driver                          |
|  +---------------------+  +--------------------+  +------------------+  |
|  | Superblock Parser   |  | Inode Table Reader |  | Extent Tree Map  |  |
|  +---------------------+  +--------------------+  +------------------+  |
+-------------------------------------------------------------------------+
                                    |
                                    v
+-------------------------------------------------------------------------+
|                     AHCI SATA Storage Block Device                      |
+-------------------------------------------------------------------------+
```

---

## 2. Superblock Structure & Validation

Upon mounting a block storage device partition, the driver inspects Sector 2 (offset `1024` bytes) for the `Ext4Superblock` header:

| Offset (Bytes) | Field | Type | Description |
| :---: | :--- | :---: | :--- |
| `0x00` | `s_inodes_count` | `u32` | Total inode count in filesystem |
| `0x04` | `s_blocks_count` | `u32` | Total block count in filesystem |
| `0x18` | `s_log_block_size` | `u32` | Block size = `1024 << s_log_block_size` |
| `0x38` | `s_magic` | `u16` | Superblock Magic Identifier (`0xEF53`) |

### Validation Code

```rust
pub const EXT4_SUPER_MAGIC: u16 = 0xEF53;

pub fn validate_superblock(magic: u16) -> bool {
    magic == EXT4_SUPER_MAGIC
}
```

---

## 3. Inode Table Reading & Extent Trees

Files and directories in EXT4 are indexed via Inode Table entries:

*   **Inode Structure**: Each 256-byte Inode structure contains file type bitmasks (`i_mode`), 32-bit byte size (`i_size`), access timestamps (`i_atime`, `i_mtime`), and 60 bytes of Extent Tree block mapping (`i_block`).
*   **Extent Tree Mapping**: EXT4 replaces traditional indirect block pointers with B-tree extent trees. Extent headers (`0xF30A`) map contiguous logical file blocks directly to physical disk sector addresses.

---

## 4. Kernel APIs

*   `pub fn init() -> Result<(), &'static str>`: Scans and mounts active EXT4 partitions registered on AHCI block devices.
*   `pub fn read_inode(inode_num: u32) -> Result<(), &'static str>`: Queries the Block Group Descriptor Table, locates the target inode, and traverses its extent tree mapping.
