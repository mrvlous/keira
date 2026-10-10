<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Filesystem & Storage Commands

The `fs` command suite provides low-level Virtual Filesystem (VFS) operations, storage volume configuration and block device management.

> [!NOTE]
> **Pure Kernel Demarcation**: Application-level file editing, viewing, copying, directory creation and removal are handled strictly in Ring 3 userspace via freestanding binaries (`/bin/cat.elf`, `/bin/ls.elf`) or the unprivileged POSIX shell (`sh`).

---

## Command Reference

| Command | Syntax | Description | Flags / Options |
| :--- | :--- | :--- | :--- |
| `disk` | `disk` | Display block storage devices and partition tables | `-h, --help` |
| `drives` | `drives` | List all active filesystem mounts and volume labels | `-h, --help` |
| `use` | `use <mount>` | Switch active default filesystem volume | `-h, --help` |
| `ramdisk` | `ramdisk` | Inspect or configure RAM-backed block disk | `-h, --help` |
| `initrd` | `initrd` | Inspect loaded RAM disk (initrd) archive entries | `-h, --help` |
| `ext4` | `ext4` | Display ext4 superblock metadata, block groups and inodes | `-h, --help` |
| `lvm` | `lvm` | Inspect Logical Volume Manager (LVM) volume groups | `-h, --help` |
| `raid` | `raid` | Query software RAID array topology and parity status | `-h, --help` |
| `swap` | `swap` | Display swap space partitions and page backing metrics | `-h, --help` |
| `sync` | `sync` | Flush dirty filesystem cache sectors to physical disk | `-h, --help` |
| `cd` / `go` | `cd [path]` / `go [path]` | Change current working directory | `-h, --help` |
