<!-- SPDX-License-Identifier: GPL-2.0-only -->

# `wc.elf` Freestanding Core Utility

The `wc.elf` executable (`/bin/wc.elf` and `/bin/wc`) is a freestanding Ring 3 core utility that counts newline characters, words, bytes, and characters in files or standard input streams.

---

## 1. Architectural Purpose

As part of Keira Milestone 37 (Text Processing Pipeline Suite), `wc.elf` provides stream quantification and metric measurement:

1. **Ring 3 Privilege Isolation**: Executes entirely in unprivileged userspace, interacting with the system exclusively through standard POSIX file descriptor system calls.
2. **Multi-Metric Text Counting**: Tracks newline counts, whitespace-delimited word counts, and byte tallies simultaneously in a single streaming pass.
3. **Pipelined Analytics**: Enables downstream metrics calculations in shell pipelines (e.g. counting lines of process listings, active sockets, or matched patterns).

---

## 2. Command-Line Options

| Option | Long Option | Description |
| :--- | :--- | :--- |
| `-l` | `--lines` | Print newline counts only |
| `-w` | `--words` | Print word counts only |
| `-c` | `--bytes` | Print byte counts only |
| `-m` | `--chars` | Print character counts only (equivalent to `-c` in ASCII/8-bit mode) |
| `-h` | `--help` | Display usage summary and exit with status 0 |

---

## 3. Usage Examples

### File Metrics

```bash
# Default metric display (lines, words, bytes, filename):
keira:/# wc /etc/hostname
1 1 6 /etc/hostname

# Counting lines only:
keira:/# wc -l /etc/kernel.cfg
4 /etc/kernel.cfg
```

### Pipeline Metrics

```bash
# Counting active processes via ps pipeline:
keira:/# ps | wc -l
7

# Counting byte size of an echo output:
keira:/# echo "Keira Pipeline" | wc -c
15
```

---

## 4. Technical Specifications

- **Source Path**: `userland/bin/wc/main.c`
- **Binary Locations**: `/bin/wc.elf`, `/bin/wc`
- **Privilege Level**: Ring 3 unprivileged
- **Supported Architectures**: `x86_64` (64-bit Long Mode), `i686` (32-bit Protected Mode)
- **Standard Library Dependency**: `libc.a` (freestanding minimal C runtime)
