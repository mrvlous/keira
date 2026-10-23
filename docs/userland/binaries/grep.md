<!-- SPDX-License-Identifier: GPL-2.0-only -->

# `grep.elf` Freestanding Core Utility

The `grep.elf` executable (`/bin/grep.elf` and `/bin/grep`) is a freestanding Ring 3 core utility that searches input files or standard input streams for lines matching a specified pattern string.

---

## 1. Architectural Purpose

As part of Keira Milestone 37 (Text Processing Pipeline Suite), `grep.elf` empowers the unprivileged userspace environment with pattern matching and filtering:

1. **Ring 3 Privilege Isolation**: Executes entirely in unprivileged userspace, performing file I/O strictly via standard POSIX system calls (`sys_open`, `sys_read`, `sys_write`, `sys_close`).
2. **Stream Filtering & Pipeline Consumption**: Accepts piped data from stdin or reads directly from specified file paths, emitting matching lines to stdout.
3. **Comprehensive Pattern Flags**: Supports case-insensitive searching (`-i`), line numbering (`-n`), inverted match selection (`-v`), and match count aggregation (`-c`).

---

## 2. Command-Line Options

| Option | Long Option | Description |
| :--- | :--- | :--- |
| `-i` | `--ignore-case` | Perform case-insensitive matching |
| `-v` | `--invert-match` | Select non-matching lines |
| `-c` | `--count` | Suppress normal output; print total count of matching lines |
| `-n` | `--line-number` | Prefix each line of output with its 1-based line number |
| `-h` | `--help` | Display usage summary and exit with status 0 |

---

## 3. Usage Examples

### Pattern Matching in Files

```bash
# Searching for a keyword in a configuration file:
keira:/# grep KERNEL /etc/kernel.cfg
KERNEL_NAME=keira
KERNEL_VERSION=0.7.0
KERNEL_ARCH=x86_64

# Case-insensitive search with line numbering:
keira:/# grep -i -n root /var/log/boot.log
2: [    0.000010] Root filesystem mounted at / (FAT16)
```

### Pipeline Filtering

```bash
# Filtering command output through a pipeline:
keira:/# dmesg | grep -i bootloader
[    0.000000] Keira Kernel v0.7.0 (x86_64) bootloader handover completed

# Inverted matching in pipelines:
keira:/# cat /etc/kernel.cfg | grep -v USERLAND
KERNEL_NAME=keira
KERNEL_VERSION=0.7.0
KERNEL_ARCH=x86_64
```

---

## 4. Technical Specifications

- **Source Path**: `userland/bin/grep/main.c`
- **Binary Locations**: `/bin/grep.elf`, `/bin/grep`
- **Privilege Level**: Ring 3 unprivileged
- **Supported Architectures**: `x86_64` (64-bit Long Mode), `i686` (32-bit Protected Mode)
- **Standard Library Dependency**: `libc.a` (freestanding minimal C runtime)
