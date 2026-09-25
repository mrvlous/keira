<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Filesystem & Storage Commands

The `fs` command suite provides Virtual Filesystem (VFS) operations, storage volume configuration, and file manipulation.

---

## Command Reference

| Command | Syntax | Description | Flags / Options |
| :--- | :--- | :--- | :--- |
| `list` | `list [path]` | List files and subdirectories with sizes and permissions | `-h, --help` |
| `go` | `go [path]` | Change current working directory | `-h, --help` |
| `view` | `view <file>` | Display text file contents to console | `-h, --help` |
| `create` | `create <file>` | Create a new empty regular file | `-h, --help` |
| `folder` | `folder <dir>` | Create a new directory | `-h, --help` |
| `delete` | `delete <path>` | Remove an existing file or empty directory | `-h, --help` |
| `copy` | `copy <src> <dst>` | Duplicate a file to a new path | `-h, --help` |
| `move` | `move <src> <dst>` | Rename or move a file | `-h, --help` |
| `write` | `write <file> <text>` | Append raw text string to file | `-h, --help` |
| `edit` | `edit <file>` | Launch visual in-terminal text editor | `-h, --help` |
| `fileinfo` | `fileinfo <file>` | Display inode details, sector addresses, and file metadata | `-h, --help` |
| `search` | `search <text> [path]`| Search for substring matches across files | `-h, --help` |
| `disk` | `disk` | Display block storage devices and partition tables | `-h, --help` |
| `drives` | `drives` | List all active filesystem mounts and volume labels | `-h, --help` |
| `use` | `use <mount>` | Switch active default filesystem volume | `-h, --help` |
| `ramdisk` | `ramdisk` | Inspect or configure RAM-backed block disk | `-h, --help` |
| `ext4` | `ext4` | Display ext4 superblock metadata, block groups, and inodes | `-h, --help` |
| `lvm` | `lvm` | Inspect Logical Volume Manager (LVM) volume groups | `-h, --help` |
| `raid` | `raid` | Query software RAID array topology and parity status | `-h, --help` |
| `sync` | `sync` | Flush dirty filesystem cache sectors to physical disk | `-h, --help` |
| `wipe` | `wipe <drive>` | Zero-fill and format storage block device | `-h, --help` |
