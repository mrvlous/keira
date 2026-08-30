<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Filesystem & Storage Shell Commands

This document details all native commands in Keira Kernel related to file operations, text editing, directory navigation, and filesystem manipulation.

---

## Command Reference Table

| Command | Syntax | Status | Description |
| :--- | :--- | :--- | :--- |
| `list` | `list [path]` | `[Active]` | List files and directories in current or specified path |
| `view` | `view <file>` | `[Active]` | Display text content of a file to the console |
| `edit` | `edit <file>` | `[Active]` | Launch full-screen interactive `kvi` modal text editor |
| `create` | `create <file> [text]` | `[Active]` | Create a new file on FAT16 filesystem |
| `delete` | `delete <path>` | `[Active]` | Delete a file or directory on FAT16 filesystem |
| `copy` | `copy <src> <dst>` | `[Active]` | Copy a file from source to destination path |
| `move` | `move <src> <dst>` | `[Active]` | Move or rename a file or directory |
| `folder` | `folder <dir>` | `[Active]` | Create a new directory on FAT16 filesystem |
| `write` | `write <file> <text>` | `[Active]` | Append or write string data to an existing FAT16 file |
| `drives` | `drives` | `[Active]` | Scan and display detected ATA/AHCI block storage drives |
| `use` | `use <drive_id>` | `[Active]` | Mount and select active block drive partition context |
| `disk` | `disk` | `[Active]` | Query primary storage drive geometry, BPB, and FAT16 layout |
| `ramdisk` | `ramdisk [info \| clear \| dump]` | `[Active]` | Inspect in-memory USTAR RAM disk initialization structures |
| `initrd` | `initrd [-c]` | `[Active]` | Traverse and list files inside the boot USTAR RAM disk archive |
| `fileinfo` | `fileinfo <path>` | `[Active]` | Display FAT16 inode metadata, starting cluster, and file size |
| `ext4` | `ext4 [info \| inodes]` | `[Preview]` | Inspect native Linux EXT4/EXT2 filesystem superblock prototype |

---

## Detailed Usage

### `edit <file>` (Interactive `kvi` Text Editor)
Keira Kernel includes `kvi`, an in-kernel text editor featuring:
* Full-screen editing with cursor navigation (Arrow keys).
* Real-time line numbers, status bar, and syntax colors.
* Keyboard shortcuts:
  * `Ctrl+S` / `F2`: Save file to FAT16 storage.
  * `Ctrl+Q` / `F10`: Exit editor back to shell.
  * `Ctrl+X`: Cut line into cut buffer.
  * `Ctrl+V`: Paste line from cut buffer.
  * `Ctrl+F`: In-editor text search.

```bash
keira> edit /config/sys/os-release
```

### `list [path]`
Displays directory entries, file types (`[file]`, `[dir]`), file sizes in bytes, and modification attributes:
```bash
keira> list /system/bin
  [file] kcc.elf        (45056 bytes)
  [file] shell.elf      (12288 bytes)
  [file] edit.elf       (8192 bytes)
```
