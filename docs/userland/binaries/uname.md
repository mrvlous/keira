<!-- SPDX-License-Identifier: GPL-2.0-only -->

# `uname.elf` Freestanding Core Utility

The `uname.elf` executable (`/bin/uname.elf` and `/bin/uname`) is a freestanding Ring 3 core utility that prints system information and kernel telemetry.

---

## 1. Architectural Purpose

System identity reporting is handled cleanly in userspace:

1. **Standard Compatibility**: Provides standard POSIX arguments (`-a`, `-s`, `-n`, `-r`, `-v`, `-m`) for script portability.
2. **Dynamic ProcFS Integration**: Reads kernel name, release version, and build date from `/proc/version`.
3. **Hostname Synchronization**: Reads current network hostname dynamically from `/etc/hostname`.

---

## 2. Command-Line Options

| Option | Long Option | Description |
| :--- | :--- | :--- |
| `-a` | `--all` | Print all system information in sequence |
| `-s` | `--kernel-name` | Print the operating system / kernel name (default) |
| `-n` | `--nodename` | Print the network node hostname |
| `-r` | `--kernel-release` | Print the kernel release version |
| `-v` | `--kernel-version` | Print the kernel build timestamp |
| `-m` | `--machine` | Print the hardware machine architecture |
| `-h` | `--help` | Displays usage summary and exits with status 0 |

---

## 3. Usage Examples

```bash
# Print default kernel name:
keira:/# uname
Keira

# Print complete system identity:
keira:/# uname -a
Keira keira 0.7.0 Keira Monolithic Kernel x86_64

# Print machine architecture:
keira:/# uname -m
x86_64
```

---

## 4. Technical Specifications

- **Source Path**: `userland/bin/uname/main.c`
- **Binary Locations**: `/bin/uname.elf`, `/bin/uname`
- **Information Sources**: `/proc/version`, `/etc/hostname`
- **Privilege Level**: Ring 3 unprivileged
