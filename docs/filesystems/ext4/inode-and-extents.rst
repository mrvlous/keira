.. SPDX-License-Identifier: GPL-2.0-only


Inode Parsing & Extent Tree Traversal
=====================================

Traverses and allocates 48-bit block extent trees (with contiguous extent coalescing and depth-1 promotion) backed by physical block and inode bitmap allocators in ``crates/fs/src/ext4/``.
