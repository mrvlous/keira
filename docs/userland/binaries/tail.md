<!-- SPDX-License-Identifier: GPL-2.0-only -->

# `tail.elf` Freestanding Core Utility

The `tail.elf` executable (`/bin/tail.elf` and `/bin/tail`) is a freestanding Ring 3 core utility that outputs the last part (default 10 lines, or a specified number of lines or bytes) of files or standard input streams.

---

## 1. Architectural Purpose

As part of Keira Milestone 37 (Text Processing Pipeline Suite), `tail.elf` provides suffix extraction and log inspection capabilities:

1. **Ring 3 Privilege Isolation**: Executes entirely in unprivileged userspace, reading input via `sys_read` and emitting stream data to stdout.
2. **Circular Buffer Architecture**: Utilizes an in-memory circular line pointer ring buffer to capture the terminal lines of streaming pipelines without requiring unbounded heap allocation.
3. **Log & Stream Diagnostics**: Ideal for monitoring the most recent events in system logs (`/var/log/boot.log`, `/var/log/system.log`) or truncating pipeline tails.

---

## 2. Command-Line Options

| Option | Long Option | Description |
| :--- | :--- | :--- |
| `-n <N>` | `--lines=<N>` | Print the last N lines (default: 10) |
| `-c <N>` | `--bytes=<N>` | Print the last N bytes |
| `-q` | `--quiet` | Never print file header banners |
| `-v` | `--verbose` | Always print file header banners |
| `-h` | `--help` | Display usage summary and exit with status 0 |

---

## 3. Usage Examples

### Slicing File Suffixes

```bash
# Outputting the last 2 lines of a file:
keira:/# tail -n 2 /etc/kernel.cfg
KERNEL_ARCH=x86_64
USERLAND=1

# Extracting the last 15 bytes:
keira:/# tail -c 15 /etc/hostname
keira
```

### Log Inspection

```bash
# Viewing the most recent boot entries:
keira:/# tail -n 1 /var/log/boot.log
[    0.000010] Root filesystem mounted at / (FAT16)
```

---

## 4. Technical Specifications

- **Source Path**: `userland/bin/tail/main.c`
- **Binary Locations**: `/bin/tail.elf`, `/bin/tail`
- **Privilege Level**: Ring 3 unprivileged
- **Supported Architectures**: `x86_64` (64-bit Long Mode), `i686` (32-bit Protected Mode)
- **Standard Library Dependency**: `libc.a` (freestanding minimal C runtime)
