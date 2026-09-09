<!-- SPDX-License-Identifier: GPL-2.0-only -->

# EXT4 Filesystem Driver & Extent Navigation

This document specifies the fourth extended filesystem (EXT4) architecture, superblock parameters, inode extent tree navigation, directory traversal, and bare-metal implementation in Keira Kernel.

---

## EXT4 Extent Tree & Directory Traversal Architecture

```mermaid
graph TD
    Superblock["Superblock (1024 bytes @ Offset 1024)<br/>Magic: 0xEF53 | Inode Size: 256B | Block Size: 4096B"] --> BlockGroup["Block Group Descriptor Table<br/>Flex-BG & 64-Bit Addressing"]
    BlockGroup --> InodeTable["Inode Table Allocation<br/>Root Inode: #2 (Mode: 0o755)"]
    InodeTable --> ExtentHeader["Extent Tree Header (eh_magic = 0xF30A)<br/>Depth: eh_depth | Entries: eh_entries"]
    ExtentHeader --> ExtentNode{"eh_depth == 0?"}
    ExtentNode -->|Leaf (eh_depth=0)| ExtentLeaf["Direct Extents (ee_block -> ee_start LBA)"]
    ExtentNode -->|Index (eh_depth>0)| ExtentIndex["Internal Index Nodes (ei_leaf LBA)"]
    ExtentLeaf --> DataBlock["Physical Storage Blocks (4096-byte Payload)"]
    DataBlock --> DirEntry["Directory Entries: Ext4DirEntry<br/>inode | rec_len | name_len | file_type | name"]
```

---

## Superblock & Inode Specifications

| Parameter | Value / Representation | Description |
| :--- | :--- | :--- |
| **Superblock Magic** | `0xEF53` | Standard EXT2 / EXT3 / EXT4 filesystem signature |
| **Block Size** | 4096 bytes | Computed as $1024 \times 2^{\text{s\_log\_block\_size}}$ |
| **Inode Size** | 256 bytes | Standard modern Linux EXT4 inode structure |
| **Extent Header Magic** | `0xF30A` | Extent leaf/node signature in `i_block` |
| **Root Inode** | `#2` | Root directory `/` inode |
| **Feature Flags** | `COMPAT_DIR_INDEX`, `INCOMPAT_EXTENTS`, `64BIT`, `FLEX_BG` | High-performance extents and 64-bit block addressing |

---

## Directory Entry Structure (`Ext4DirEntry`)

Linear directory records in EXT4 data blocks adhere to the standard Linux layout:

```rust
#[repr(C, packed)]
pub struct Ext4DirEntry {
    pub inode: u32,
    pub rec_len: u16,
    pub name_len: u8,
    pub file_type: u8,
    // Variable length name payload followed by padding
}
```

Supported directory file types:
* `0x00`: Unknown
* `0x01`: Regular File (`REG`)
* `0x02`: Directory (`DIR`)
* `0x03`: Character Device (`CHR`)
* `0x04`: Block Device (`BLK`)
* `0x05`: FIFO (`FIFO`)
* `0x06`: Socket (`SOCK`)
* `0x07`: Symbolic Link (`SYMLINK`)

---

## Core API (`crates/fs/src/ext4/`)

* `mount() -> Result<(), &'static str>`: Mounts default EXT4 partition `/system/dev/sda2` and verifies superblock magic.
* `get_ext4_superblock() -> Option<&'static Ext4Superblock>`: Returns immutable reference to active superblock.
* `read_inode(inode_nr: u32) -> Result<Ext4Inode, &'static str>`: Reads and parses 256-byte inode structure from disk.
* `lookup_path(path: &str) -> Result<(u32, Ext4Inode), &'static str>`: Resolves hierarchical path (e.g., `/system/config.sys`) to its inode number and descriptor.
* `read_file_content(path: &str, out: &mut [u8]) -> Result<usize, &'static str>`: Reads data bytes from a regular file across its extent tree blocks.
* `get_dir_entries(dir_inode: u32) -> Option<&'static [Ext4DirEntry]>`: Enumerates parsed directory entries.

---

## Shell Command Usage (`ext4`)

The `ext4` command provides interactive inspection and testing of native EXT4 filesystems:

```bash
# Display overall mount status and storage capacity
keira> ext4 status
Native Linux EXT4 Filesystem Driver [Active]
  Status      : Mounted (/system/dev/sda2)
  Storage     : 256 MB Total (192 MB Free)
  Inodes      : 65536 total (61440 free)
  Block Size  : 4096 bytes
  Features    : Extents, 64-Bit, Flex-BG, Dir-Index

# Inspect superblock parameters and compatibility flags
keira> ext4 info

# Traverse directory contents
keira> ext4 ls /
EXT4 Directory [/] (Inode #2):
  [DIR] Inode #2        .
  [DIR] Inode #2        ..
  [DIR] Inode #11       system
  [REG] Inode #12       boot.cfg
  [REG] Inode #13       vmlinuz

# Read file contents over extent tree
keira> ext4 cat /boot.cfg
--- Content of /boot.cfg ---
TIMEOUT=5
DEFAULT=keira
TITLE=Keira Kernel 0.1.0 LTS

# Run automated self-test
keira> ext4 test
[TEST] Executing Native Linux EXT4 Driver Self-Test...
  1. Verified superblock magic (0xEF53) - OK
  2. Calculated block group descriptors (8 BGs) - OK
  3. Validated root inode #2 mode (0o755 directory) - OK
  4. Traversed extent tree (Physical LBA: 32768) - OK
  5. Read regular file /boot.cfg over extents (64 bytes) - OK
[PASS] Native Linux EXT4 Filesystem Driver operational.
```
