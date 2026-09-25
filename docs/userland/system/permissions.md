<!-- SPDX-License-Identifier: GPL-2.0-only -->

# File System Modes & Access Boundaries

Keira enforces filesystem safety through VFS route permissions, node type validations, and Ring 3 boundary isolation.

---

## VFS Node Kinds

Each VFS inode distinguishes file types and access characteristics:
* **Regular File**: Byte-stream storage on FAT16, EXT4, or Initrd backends.
* **Directory**: Hierarchical path container with cluster or record resolution.
* **Character Device**: Streaming hardware endpoints under `/system/dev/` (e.g. `tty0`, `urandom`).
* **Block Device**: Block-addressed storage volumes (e.g. `ram0`, `sata0`, `nvme0`).
* **Pseudo-Nodes**: Dynamic kernel state telemetry endpoints under `/system/proc/` (e.g. `uptime`, `meminfo`).
