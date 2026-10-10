<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Filesystem & Storage Commands

The `fs` command suite provides low-level Virtual Filesystem (VFS) operations, storage volume configuration and block device management.

> [!NOTE]
> **Pure Kernel Demarcation**: Application-level file editing, viewing, copying, directory creation and removal are handled strictly in Ring 3 userspace via freestanding binaries (`/bin/cat.elf`, `/bin/ls.elf`) or the unprivileged POSIX shell (`sh`).

---

## Command Reference

| Command | Syntax | Description | Flags / Options |
| :--- | :--- | :--- | :--- |
| `list` | `list [path]` | List files and subdirectories with sizes and permissions | `-h, --help` |
| `go` / `cd` | `go [path]` / `cd [path]` | Change current working directory (canonical alias: `cd`) | `-h, --help` |
| `fileinfo` | `fileinfo <file>` | Display inode details, sector addresses and file metadata | `-h, --help` |
| `search` | `search <text> [path]`| Search for substring matches across files | `-h, --help` |
| `disk` | `disk` | Display block storage devices and partition tables | `-h, --help` |
| `drives` | `drives` | List all active filesystem mounts and volume labels | `-h, --help` |
| `use` | `use <mount>` | Switch active default filesystem volume | `-h, --help` |
| `ramdisk` | `ramdisk` | Inspect or configure RAM-backed block disk | `-h, --help` |
| `ext4` | `ext4` | Display ext4 superblock metadata, block groups and inodes | `-h, --help` |
| `lvm` | `lvm` | Inspect Logical Volume Manager (LVM) volume groups | `-h, --help` |
| `raid` | `raid` | Query software RAID array topology and parity status | `-h, --help` |
| `sync` | `sync` | Flush dirty filesystem cache sectors to physical disk | `-h, --help` |
| `wipe` | `wipe <drive>` | Zero-fill and format storage block device | `-h, --help` |
