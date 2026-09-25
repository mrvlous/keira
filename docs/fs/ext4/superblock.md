<!-- SPDX-License-Identifier: GPL-2.0-only -->

# EXT4 Filesystem Superblock & Extents

Keira incorporates a read-only EXT4 driver (`crates/fs/src/ext4/`) capable of mounting Linux partitions, parsing inodes, and traversing B-tree extent trees.

---

## 1. EXT4 Superblock Architecture

Located at byte offset `1024` from partition start:
* **Magic Number**: `0xEF53` at offset `0x38`.
* **Block Size**: $1024 \times 2^{\text{s\_log\_block\_size}}$ (typically 4096 bytes).
* **Blocks Per Group**: Determines block group boundaries across the disk.
* **Inodes Per Group**: Number of inode table entries per block group descriptor.

---

## 2. Extent Tree Traversal

EXT4 replaces indirect block pointers with extent trees for high-speed linear data streaming. An inode's `i_block[0..14]` contains an `ext4_extent_header`:

```text
+-------------------------------------------------------+
| ext4_extent_header (magic=0xF30A, entries, max, depth)|
+-------------------------------------------------------+
| Index Node (depth > 0)    | or Leaf Extent (depth == 0)
| - Logical Block (32-bit)  |    - Logical Block (32-bit)
| - Physical Leaf Block     |    - Block Count (16-bit)
|                           |    - Physical Block (48-bit)
+---------------------------+---------------------------+
```

Leaf extents describe contiguous runs of up to 32,768 physical blocks with zero metadata overhead.
