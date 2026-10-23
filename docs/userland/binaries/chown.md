<!-- SPDX-License-Identifier: GPL-2.0-only -->

# `chown.elf` Freestanding Core Utility

The `chown.elf` executable (`/bin/chown.elf` and `/bin/chown`) is a freestanding Ring 3 core utility that modifies file and directory owner and group identity according to POSIX Discretionary Access Control (DAC) standards.

---

## 1. Architectural Purpose

File ownership assignment is handled via Syscall 98 (`SYS_CHOWN`):

1. **User and Group Syntax**: Accepts `owner:group`, `owner`, `:group` syntax with support for numerical identifiers as well as named aliases (`root`).
2. **Superuser Privilege Verification**: In conformance with POSIX security semantics, the kernel verifies that the caller has effective superuser privileges (EUID 0) before modifying ownership metadata.
3. **Recursive Directory Traversal**: Supports changing ownership across whole directory subtrees with `-R` (`--recursive`).
4. **Multi-User Isolation**: Establishes multi-tenant environment support where unprivileged users can be assigned ownership over workspaces and data files.

---

## 2. Command-Line Options

| Option | Long Option | Description |
| :--- | :--- | :--- |
| `-R` | `--recursive` | Operate on files and directories recursively |
| `-v` | `--verbose` | Output a diagnostic confirmation message for every file processed |
| `-h` | `--help` | Displays usage summary and exits with status 0 |

---

## 3. Syntax Formats

- `owner:group`: Sets both owner UID and group GID (e.g. `1000:1000`, `root:root`, `0:0`).
- `owner`: Sets owner UID while leaving group GID unchanged (e.g. `1000`, `root`).
- `:group`: Sets group GID while leaving owner UID unchanged (e.g. `:1000`, `:root`).

---

## 4. Usage Examples

```bash
# Transfer ownership of a file to user 1000:
keira:/# touch /tmp/userfile.txt
keira:/# chown 1000:1000 /tmp/userfile.txt
keira:/# ls -l /tmp/userfile.txt
-rw-r--r--  1 1000 1000        0 userfile.txt

# Reclaim root ownership:
keira:/# chown root:root /tmp/userfile.txt
keira:/# ls -l /tmp/userfile.txt
-rw-r--r--  1    0    0        0 userfile.txt

# Recursively change directory ownership with verbose feedback:
keira:/# chown -Rv 1000:1000 /tmp/dir
ownership of '/tmp/dir' changed to 1000:1000
ownership of '/tmp/dir/file.txt' changed to 1000:1000
```

---

## 5. Technical Specifications

- **Source Path**: `userland/bin/chown/main.c`
- **Binary Locations**: `/bin/chown.elf`, `/bin/chown`
- **System Call**: Syscall 98 (`SYS_CHOWN`)
- **Privilege Level**: Ring 3 unprivileged interface (kernel requires EUID 0)
- **Dependencies**: Freestanding `libc.a` (`<dirent.h>`, `<errno.h>`, `<stdio.h>`, `<stdlib.h>`, `<string.h>`, `<sys/stat.h>`, `<unistd.h>`)
