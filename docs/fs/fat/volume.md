<!-- SPDX-License-Identifier: GPL-2.0-only -->

# FAT12 / FAT16 / FAT32 Filesystem Driver

Keira implements a robust, spec-compliant FAT filesystem driver (`crates/fs/src/fat/`) supporting volume mounting, cluster chain traversal, file creation, and data streaming.

---

## 1. BIOS Parameter Block (BPB) Layout

The first sector (LBA 0 or partition boot sector) contains the BPB:

| Offset | Field Name | Size | Description |
| :--- | :--- | :--- | :--- |
| `0x0B` | `bytes_per_sector` | 2 bytes | Typically 512 bytes |
| `0x0D` | `sectors_per_cluster` | 1 byte | Power of 2 (1, 2, 4, 8, 16, 32, 64) |
| `0x0E` | `reserved_sectors` | 2 bytes | Sectors before first FAT (e.g. 32 for FAT32) |
| `0x10` | `num_fats` | 1 byte | Count of FAT tables (usually 2 for redundancy) |
| `0x11` | `root_entries` | 2 bytes | Root directory entry count (0 for FAT32) |
| `0x13` | `total_sectors_16` | 2 bytes | 16-bit total sector count (if < 32MB) |
| `0x16` | `sectors_per_fat_16` | 2 bytes | Sectors per FAT (FAT12/16) |
| `0x20` | `total_sectors_32` | 4 bytes | 32-bit total sector count |
| `0x24` | `sectors_per_fat_32` | 4 bytes | Sectors per FAT (FAT32) |
| `0x2C` | `root_cluster` | 4 bytes | Starting cluster of root directory (FAT32, usually 2) |

---

## 2. Cluster Mathematics

To translate a cluster number to a logical block address (LBA):
$$\text{FAT Start LBA} = \text{Partition Base} + \text{reserved\_sectors}$$
$$\text{Data Start LBA} = \text{FAT Start LBA} + (\text{num\_fats} \times \text{sectors\_per\_fat})$$
$$\text{Cluster LBA} = \text{Data Start LBA} + ((\text{Cluster} - 2) \times \text{sectors\_per\_cluster})$$

---

## 3. FAT Chain Traversal

Entries in the File Allocation Table form singly-linked lists:
* `0x00000000`: Free cluster.
* `0x00000002`--`0x0FFFFFEF`: Pointer to next cluster in file.
* `0x0FFFFFF8`--`0x0FFFFFFF`: End-of-File (EOF) marker.
* `0x0FFFFFF7`: Bad cluster indicator.
