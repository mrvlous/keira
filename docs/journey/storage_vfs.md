<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Milestone 4: Storage, Filesystems & Sector Caching

This journal entry details the creation of the unified Virtual Filesystem (VFS), FAT16 cluster chaining, and in-memory LRU block caching in Keira Kernel.

---

## Engineering Challenges

1. **Storage Device Diversity**: Supporting IDE ATA, AHCI SATA, NVMe, and RAM disks under a single consistent file API.
2. **I/O Bottlenecks**: Raw disk sector reads are slow. Without an intelligent caching layer, reading FAT directory clusters repeatedly degrades system responsiveness.

---

## Solutions & Design Choices

* **Trait-Based VFS Layer**: Decoupled filesystem implementations from physical drivers using abstract `FileSystem` and `BlockDevice` traits.
* **16-Slot LRU Sector Cache**: Built a write-through sector cache that caches hot FAT tables and directory entries in memory with monotonic clock eviction.
