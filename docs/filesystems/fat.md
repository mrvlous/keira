<!--
SPDX-License-Identifier: GPL-2.0-only

Keira Kernel - Operating System Kernel
Copyright (C) 2026 Moh. Ananda Firmansyah Putra
-->

# FAT Filesystem Subsystem

This document details the layout, data structures, directory traversal, and cluster management of the FAT12/16/32 filesystem driver implemented in Keira Kernel.

## 1. FAT Data Structures and Volume Mounting
The FAT subsystem ([fat/](../../kernel/src/fs/fat)) supports FAT12, FAT16, and FAT32 disk layouts.

### BIOS Parameter Block (BPB)
During mounting (`fat::volume::mount`), the driver reads sector 0 of the partition to parse the BPB metadata defined in [types.rs](../../kernel/src/fs/fat/types.rs):
*   `bytes_per_sector`: Sector size in bytes (usually 512).
*   `sectors_per_cluster`: Power-of-two multiplier defining cluster size.
*   `reserved_sector_count`: Sectors before the first File Allocation Table (FAT).
*   `num_fats`: Number of redundant FAT tables (typically 2).
*   `total_sectors_16` / `total_sectors_32`: Total sectors on the volume.
*   `fat_size_16` / `fat_size_32`: Sectors occupied by a single FAT table.

From these parameters, the driver calculates:
*   **FAT Start Sector**: `reserved_sector_count`.
*   **Data Start Sector**: `reserved_sector_count + (num_fats * fat_size)`.
*   **Total Clusters**: Used to differentiate between FAT12 (less than 4086), FAT16 (less than 65526), and FAT32 (65526 or more).

---

## 2. Cluster Chain Resolution
Files and directories are stored in chains of clusters. The FAT table acts as a linked list to map these chains.

The driver ([cluster.rs](../../kernel/src/fs/fat/cluster.rs)) implements the following cluster operations:
*   **Read Next Cluster**: Given current cluster `N`, it calculates the sector offset in the FAT table, reads that sector, and extracts entry `N`.
    *   FAT12: Reads 1.5 bytes.
    *   FAT16: Reads 2 bytes (uint16).
    *   FAT32: Reads 4 bytes (uint32).
*   **End of Chain**: Values like `0xFFF` (FAT12), `0xFFFF` (FAT16), or `0x0FFFFFFF` (FAT32) indicate the end of the file.
*   **Allocation**: To extend a file, the driver searches the FAT table for an entry containing 0, sets it to the End-of-Chain marker, and links the previous cluster to this new entry.

---

## 3. Directory Traversal and LFN Support
Directories contain a list of 32-byte entries. The driver ([dir.rs](../../kernel/src/fs/fat/dir.rs)) parses these entries to find files.

### Directory Entry Layout
*   **Standard 8.3 Entry**:
    *   Bytes 0-10: Short file name (padded with spaces).
    *   Byte 11: File attribute flags (Directory, Read-Only, System, Hidden, Volume ID, LFN).
    *   Bytes 26-27 (and 20-21 for FAT32): First cluster of the file data.
    *   Bytes 28-31: File size in bytes.
*   **Long File Name (LFN) Entry**:
    *   Identified by the attribute flag `0x0F` (Read-Only + Hidden + System + Volume ID).
    *   Contains UTF-16 character sequences encoding long file names. The driver parses these sequences to build the full filename before matching against target paths.

---

## 4. Path Resolution and Write API
The path parser ([path.rs](../../kernel/src/fs/fat/path.rs)) splits virtual absolute paths (e.g. `/disk/home/user/test.txt`) and traverses the directory tree:
1.  **Resolve Parent Directory**: Iteratively reads directory clusters, parsing entries to locate subdirectories until reaching the target parent directory.
2.  **File Operations**:
    *   **Read**: Walks the cluster chain of the file, reading sectors and writing their contents into a target memory buffer.
    *   **Write/Create**: Allocates a free directory entry in the parent directory, sets its short/long name, allocates initial data clusters, and writes data to disk while updating file size fields.
    *   **Append**: Appends byte arrays to existing file streams (`append_file_content`), dynamically allocating additional FAT clusters and expanding file chain entries without corrupting existing data.

---

## 5. Sector Cache (Block Cache)
To speed up FAT table lookups, cluster chain traversal, and file/directory read operations, the driver implements a static 16-slot LRU (Least Recently Used) Sector Cache in [fat.rs](../../kernel/src/fs/fat.rs).

*   **Cache Strategy**: Write-Through. All writes are immediately written to the physical/virtual storage device to prevent filesystem corruption on VM/system shutdown, but the sector is kept in the cache to accelerate subsequent reads.
*   **Cache Eviction**: LRU (Least Recently Used) based on a global clock counter.
*   **Cache Invalidation**: The entire cache is automatically cleared when mounting/initializing a new volume (via `volume::init`) to prevent stale data.
*   **Dirty Page Flushing (`sync`)**: Invoking `flush_dirty_sectors()` (or the shell command `sync`) explicitly flushes all modified sector cache pages from memory to physical storage.

---

## 6. File Protection & Metadata Inspection Commands
Keira Kernel provides native, full-word shell commands to inspect and modify FAT16 directory entry attributes without relying on Linux naming conventions:
*   **`protect <file_path> <readonly|readwrite>`**: Toggles the Read-Only attribute flag (`0x01`) on a FAT16 file entry, preventing unauthorized modifications.
*   **`fileinfo <file_path>`**: Inspects detailed file entry metadata, including file size in bytes, first cluster index, attribute bitmasks, and write protection status.
