<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Utility & Navigation Shell Commands

This document details all shell utility and navigation commands in Keira Kernel.

---

## Command Reference Table

| Command | Syntax | Status | Description |
| :--- | :--- | :--- | :--- |
| `guide` | `guide [command]` | `[Active]` | Interactive system command directory and documentation guide system |
| `help` | `help [command]` | `[Active]` | Alias route to `guide` command for interactive help |
| `history` | `history [-c] [-n <count>]` | `[Active]` | Display or clear interactive shell command history ring buffer |
| `go` | `go [path]` | `[Active]` | Change working directory on FAT16 storage with relative (`.`, `..`) and tilde (`~`) support |
| `search` | `search <pattern> [filename]` | `[Active]` | Search for lines matching text pattern in file or from stdin pipe |
| `script` | `script <filename.sh>` | `[Active]` | Read and execute shell commands sequentially from a script file |
| `wait` | `wait <milliseconds>` | `[Active]` | Suspend shell execution for specified duration using CPU `hlt` loops |
| `wipe` | `wipe` | `[Active]` | Reset VGA text buffer, clear screen contents, and home cursor |

---

## Detailed Usage

### `history`
Inspects recently entered shell commands:
```bash
keira> history
  1  system
  2  drives
  3  list /system/bin
  4  network
  5  history
```

### `go <path>`
Navigates directories across the FAT16 filesystem:
```bash
keira> go /users/admin
admin@keira:/users/admin$ go ..
admin@keira:/users$ go ~
admin@keira:/users/admin$
```
