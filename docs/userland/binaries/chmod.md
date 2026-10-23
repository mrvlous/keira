<!-- SPDX-License-Identifier: GPL-2.0-only -->

# `chmod.elf` Freestanding Core Utility

The `chmod.elf` executable (`/bin/chmod.elf` and `/bin/chmod`) is a freestanding Ring 3 core utility that modifies file and directory mode bits according to standard POSIX Discretionary Access Control (DAC) rules.

---

## 1. Architectural Purpose

File permission management is handled in userspace via Syscall 97 (`SYS_CHMOD`):

1. **Octal and Symbolic Permissions**: Supports numeric octal modes (such as `755`, `644`, `600`) as well as symbolic mode specifications (such as `+x`, `-w`, `u+rwx`, `go-w`).
2. **POSIX Access Control Enforcement**: Changes are validated against caller credentials in the kernel DAC engine, allowing only the file owner or root (EUID 0) to alter permission bits.
3. **Recursive Directory Traversal**: Supports traversing directory hierarchies with `-R` (`--recursive`) to apply mode adjustments across whole subtrees.
4. **Self-Hosting and Security Foundation**: Essential for setting executable bits on compiled binaries, securing configuration files and protecting system daemons.

---

## 2. Command-Line Options

| Option | Long Option | Description |
| :--- | :--- | :--- |
| `-R` | `--recursive` | Change files and directories recursively |
| `-v` | `--verbose` | Output a diagnostic confirmation message for every file processed |
| `-h` | `--help` | Displays usage summary and exits with status 0 |

---

## 3. Supported Modes

- **Octal Modes**: Numeric mode bitmasks up to 4 octal digits (`07777`):
  - `755`: Read, write and execute for owner; read and execute for group and others (`rwxr-xr-x`).
  - `644`: Read and write for owner; read-only for group and others (`rw-r--r--`).
  - `600`: Read and write for owner only (`rw-------`).
  - `700`: Full permissions for owner only (`rwx------`).
- **Symbolic Modes**: Clauses specifying target class (`u`, `g`, `o`, `a`), operator (`+`, `-`, `=`) and permissions (`r`, `w`, `x`):
  - `+x`: Grant execution permission to all classes.
  - `-w`: Revoke write permission from all classes.
  - `u+rwx`: Grant full permissions to file owner.
  - `go-w`: Revoke write permissions from group and others.

---

## 4. Usage Examples

```bash
# Set secure private permissions on a secret file:
keira:/# touch /tmp/secret.txt
keira:/# chmod 600 /tmp/secret.txt
keira:/# ls -l /tmp/secret.txt
-rw-------  1    0    0        0 secret.txt

# Grant executable permission to a script:
keira:/# chmod +x /tmp/script.sh
keira:/# ls -l /tmp/script.sh
-rwxr-xr-x  1    0    0       48 script.sh

# Recursively change directory mode with verbose feedback:
keira:/# chmod -Rv 755 /tmp/dir
mode of '/tmp/dir' changed to 0755
mode of '/tmp/dir/file.txt' changed to 0755
```

---

## 5. Technical Specifications

- **Source Path**: `userland/bin/chmod/main.c`
- **Binary Locations**: `/bin/chmod.elf`, `/bin/chmod`
- **System Call**: Syscall 97 (`SYS_CHMOD`)
- **Privilege Level**: Ring 3 unprivileged (kernel enforces owner/root check)
- **Dependencies**: Freestanding `libc.a` (`<dirent.h>`, `<errno.h>`, `<stdio.h>`, `<stdlib.h>`, `<string.h>`, `<sys/stat.h>`, `<unistd.h>`)
