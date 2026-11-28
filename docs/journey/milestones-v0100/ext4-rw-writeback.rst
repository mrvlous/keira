.. SPDX-License-Identifier: GPL-2.0-only


Milestone 70: Full Read-Write EXT4 Filesystem & Asynchronous [kflushd] Writeback
================================================================================

Milestone 70 promotes Keira's EXT4 driver (``crates/fs/src/ext4/``) from a read-only parser into a persistent, Linux-compatible Read-Write filesystem integrated with the unified 4 KiB page cache and ``[kflushd]`` writeback daemon.

1. Dual-Port SATA AHCI Controller & DevFS Partitioning
======================================================

The AHCI host driver (``crates/io/src/storage/ahci.rs``) enumerates multiple active SATA ports simultaneously with dedicated DMA Command List, Received FIS and PRDT buffers per slot:

* **``/dev/sda`` (``ahci0``)**: Primary FAT16 root filesystem disk image.
* **``/dev/sdb`` (``ahci1``)**: Secondary persistent 16 MiB EXT4 volume (automatically formatted via ``mkfs.ext4`` and auto-mounted at ``/mnt/ext4``).

2. Read-Write EXT4 On-Disk Metadata & Extent Engine
===================================================

* **Block Group Descriptor Table (``bgdt.rs``)**: Parses and updates free block counts, free inode counts and used directory counts per block group.
* **Block & Inode Bitmaps (``bitmap.rs``)**: Allocates and releases physical 4 KiB blocks and inodes atomically across bitmap sectors while keeping superblock summary counters synchronized.
* **Extent Tree Allocation (``inode.rs``)**: Appends contiguous physical runs into ``Ext4Extent`` leaf nodes, automatically coalescing adjacent blocks and promoting full inline ``eh_depth = 0`` nodes into depth-1 internal index trees (``Ext4ExtentIdx``) when needed.
* **Directory Entry Manipulation (``dir.rs``)**: Supports ``mkdir`` (including ``.`` and ``..`` links), file creation, linear slot splitting/coalescing and safe ``unlink``/``rmdir`` reclamation.

3. Unified Page Cache & [kflushd] Writeback Integration
=======================================================

All regular file writes through ``/mnt/ext4`` stage dirty 4 KiB pages under ``EXT4_CACHE_DEV_ID`` (device ID ``2``) in ``crates/fs/src/cache/page.rs``. Background thread ``[kflushd]`` (PID 3) and explicit ``sync``/``fsync`` system calls invoke ``writeback_page()`` and ``sync_filesystem()`` to flush dirty blocks cleanly onto physical SATA sectors, passing host ``e2fsck -fn`` integrity validation with zero errors.
