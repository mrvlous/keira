<!-- SPDX-License-Identifier: GPL-2.0-only -->

# `head.elf` Freestanding Core Utility

The `head.elf` executable (`/bin/head.elf` and `/bin/head`) is a freestanding Ring 3 core utility that outputs the first part (default 10 lines, or a specified number of lines or bytes) of files or standard input streams.

---

## 1. Architectural Purpose

As part of Keira Milestone 37 (Text Processing Pipeline Suite), `head.elf` enables prefix stream slicing:

1. **Ring 3 Privilege Isolation**: Executes entirely in unprivileged userspace, reading input via `sys_read` and emitting stream data to stdout.
2. **Flexible Truncation**: Supports line-based prefix extraction (`-n <lines>`) as well as byte-based truncation (`-c <bytes>`).
3. **Multi-File Header Headers**: When operating on multiple file arguments, cleanly renders header separators unless suppressed with quiet mode (`-q`).

---

## 2. Command-Line Options

| Option | Long Option | Description |
| :--- | :--- | :--- |
| `-n <N>` | `--lines=<N>` | Print the first N lines (default: 10) |
| `-c <N>` | `--bytes=<N>` | Print the first N bytes |
| `-q` | `--quiet` | Never print file header banners |
| `-v` | `--verbose` | Always print file header banners |
| `-h` | `--help` | Display usage summary and exit with status 0 |

---

## 3. Usage Examples

### Slicing File Prefixes

```bash
# Outputting the first 2 lines of a file:
keira:/# head -n 2 /etc/kernel.cfg
KERNEL_NAME=keira
KERNEL_VERSION=0.7.0

# Reading the first 16 bytes:
keira:/# head -c 16 /tmp/main.c
/* Keira Compreh
```

### Pipeline Truncation

```bash
# Restricting process table output to top 3 lines:
keira:/# ps | head -n 3
  PID  PPID USER     STAT   COMMAND
    0     0 root     R      kernel_shell
    1     0 root     S      /bin/sh
```

---

## 4. Technical Specifications

- **Source Path**: `userland/bin/head/main.c`
- **Binary Locations**: `/bin/head.elf`, `/bin/head`
- **Privilege Level**: Ring 3 unprivileged
- **Supported Architectures**: `x86_64` (64-bit Long Mode), `i686` (32-bit Protected Mode)
- **Standard Library Dependency**: `libc.a` (freestanding minimal C runtime)
