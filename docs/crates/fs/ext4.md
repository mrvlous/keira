<!-- SPDX-License-Identifier: GPL-2.0-only -->

# EXT4 & EXT2 Linux Filesystem Driver

Documentation for EXT4 filesystem support in [`crates/fs/src/ext4/`](../../../crates/fs/src/ext4).

## Features
- Superblock and Block Group Descriptor table parsing.
- 32-bit Inode table reading with extended attributes.
- Extent Tree (`ext4_extent_header`, `ext4_extent`, `ext4_extent_idx`) mapping for 48-bit physical block addresses.
