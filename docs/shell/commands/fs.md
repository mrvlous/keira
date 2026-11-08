<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Filesystem & Storage Commands

The `fs` command suite provides low-level block storage device and partition geometry diagnostics.

> [!NOTE]
> **Pure Kernel Demarcation**: Application-level file editing, viewing, copying, directory traversal and removal are handled strictly in Ring 3 userspace via freestanding binaries (`/bin/cat.elf`, `/bin/ls.elf`, `/bin/df.elf`) or the unprivileged POSIX shell (`sh`).

---

## Command Reference

| Command | Syntax | Description | Flags / Options |
| :--- | :--- | :--- | :--- |
| `dcache` | `dcache [-c]` | Inspect VFS in-memory directory entry (dentry) cache telemetry and hit ratios | `-c, --clear`: Invalidate all cached dentries<br>`-h, --help`: Usage info |
| `disk` | `disk` | Display block storage devices and partition tables | `-h, --help` |
| `icache` | `icache [-c]` | Inspect VFS in-memory inode cache telemetry and hit ratios | `-c, --clear`: Invalidate all cached inodes<br>`-h, --help`: Usage info |
| `mount` | `mount` | Display active VFS mount table entries and flags directly from Ring 0 | `-h, --help` |

