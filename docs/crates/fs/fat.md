<!-- SPDX-License-Identifier: GPL-2.0-only -->

# FAT12 / FAT16 / FAT32 Filesystem Driver

Documentation for FAT filesystem support in [`crates/fs/src/fat/`](../../../crates/fs/src/fat).

## Features
- Supports BIOS Parameter Block (BPB) and Extended BPB parsing.
- Cluster chain traversal and allocation bitmap management.
- 16-slot LRU sector cache for accelerated file access.
- 8.3 short filename parsing and path directory traversal.
