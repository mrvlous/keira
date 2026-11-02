<!-- SPDX-License-Identifier: GPL-2.0-only -->

# `dmesg.elf` Freestanding Core Utility

The `dmesg.elf` executable (`/bin/dmesg.elf` and `/bin/dmesg`) is a freestanding Ring 3 core utility that displays kernel diagnostic and boot ring buffer messages.

---

## 1. Architectural Purpose

Kernel event logs and boot traces are queried through live kernel memory:

1. **Live Kernel Ring Buffer Telemetry**: Invokes Syscall 44 (`SYS_SYSLOG`) with action `SYSLOG_ACTION_READ_ALL` (3) to retrieve live, ordered telemetry directly from the kernel's in-memory 32 KiB circular ring buffer.
2. **Chunked Streaming**: Emits log data to standard output, integrating cleanly with shell pipes (e.g. `dmesg | cat`).
3. **Purity**: Uses verified user-space system call interfaces without depending on static dummy log files.

---

## 2. Command-Line Options

| Option | Long Option | Description |
| :--- | :--- | :--- |
| `-h` | `--help` | Displays usage summary and exits with status 0 |

---

## 3. Usage Examples

```bash
# Display live kernel ring buffer log:
keira:/# dmesg
[     0.000000 ] x86/boot: Keira Kernel v0.7.0 booting (pure freestanding Ring 0)
[     0.000000 ] serial: 16550A UART at port 0x3f8 (irq 4, baud 115200)
[     0.000000 ] console: colour VGA+ 80x25 text console ready
[     0.000000 ] idt: 256 interrupt descriptor gates mapped
[     0.171038 ] vfs: mounted root filesystem (FAT16)

# Pipe kernel logs:
keira:/# dmesg | cat
```

---

## 4. Technical Specifications

- **Source Path**: `userland/bin/dmesg/main.c`
- **Binary Locations**: `/bin/dmesg.elf`, `/bin/dmesg`
- **Privilege Level**: Ring 3 unprivileged
- **Telemetry Vector**: Syscall 44 (`SYS_SYSLOG`)
