<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Journey Milestone 4: Storage Drivers & Virtual Filesystems

This milestone documents the layered storage architecture, from block device controllers to userland filesystem APIs.

---

## Key Achievements

1. **Block Driver Subsystem**: AHCI SATA, NVMe controller, legacy IDE, and in-memory RAM disk drivers.
2. **Virtual Filesystem (VFS)**: Mount table dispatching paths to concrete filesystem implementations.
3. **FAT12/16/32 Driver**: Cluster chain traversal, directory parsing, file creation, reading, and writing.
4. **EXT4 Reader**: Inode parsing, superblock verification, and extent tree navigation.
5. **USTAR Initrd**: Early boot ramdisk archive extraction.
