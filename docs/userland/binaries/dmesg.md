<!-- SPDX-License-Identifier: GPL-2.0-only -->

# `dmesg.elf` Freestanding Core Utility

The `dmesg.elf` executable (`/bin/dmesg.elf` and `/bin/dmesg`) is a freestanding Ring 3 core utility that displays the kernel boot messages and system event logs.

---

## 1. Architectural Purpose

Kernel event logs and boot traces are inspected through standard log file paths:

1. **Log Stream Traversal**: Reads `/var/log/system.log` and `/var/log/boot.log` sequentially.
2. **Chunked Streaming**: Emits log data to standard output in 512-byte blocks, integrating cleanly with shell pipes (e.g. `dmesg | cat`).
3. **Purity**: Avoids direct supervisor console memory poking; reads structured logs through the VFS abstraction.

---

## 2. Command-Line Options

| Option | Long Option | Description |
| :--- | :--- | :--- |
| `-h` | `--help` | Displays usage summary and exits with status 0 |

---

## 3. Usage Examples

```bash
# Display system log:
keira:/# dmesg
[0.000000] Keira Kernel boot initialized
[0.001200] ACPI 2.0 tables located
[0.004500] SMP: 4 online cores brought up
[0.010200] VFS root mounted successfully

# Pipe kernel logs:
keira:/# dmesg | cat
```

---

## 4. Technical Specifications

- **Source Path**: `userland/bin/dmesg/main.c`
- **Binary Locations**: `/bin/dmesg.elf`, `/bin/dmesg`
- **Privilege Level**: Ring 3 unprivileged
- **Log Paths**: `/var/log/system.log`, `/var/log/boot.log`
