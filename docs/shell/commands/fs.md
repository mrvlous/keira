<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Filesystem & Storage Shell Commands

This document details all native commands in Keira Kernel related to file operations, text editing, directory navigation, and filesystem manipulation.

---

## Command Reference Table

| Command | Syntax | Description |
| :--- | :--- | :--- |
| `list` | `list [path]` | List files and directories in current or specified path |
| `view` | `view <file>` | Display text content of a file to the console |
| `edit` | `edit <file>` | Launch full-screen interactive `kvi` text editor |
| `create` | `create <file> [text]` | Create a new file with optional initial text |
| `delete` | `delete <path>` | Delete a file or directory recursively |
| `copy` | `copy <src> <dst>` | Copy a file from source to destination |
| `move` | `move <src> <dst>` | Move or rename a file or directory |
| `folder` | `folder <dir>` | Create a new directory |
| `write` | `write <file> <text>` | Append or write string data to an existing file |
| `drives` | `drives` | Display mounted filesystem drives (`A:`, `B:`, `C:`) |
| `disk` | `disk [info \| read \| write]` | Query block disk geometry, sectors, and partition tables |
| `ramdisk` | `ramdisk [info \| clear \| dump]` | Inspect and manage volatile in-memory block devices |
| `initrd` | `initrd [list \| cat <file>]` | Inspect boot USTAR RAM disk archive |
| `ext4` | `ext4 [mount \| info \| list]` | Inspect EXT4 filesystem superblock and block groups |
| `use` | `use <path>` | Change working directory or drive context |

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
