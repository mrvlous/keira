<!-- SPDX-License-Identifier: GPL-2.0-only -->

# `sleep.elf` Freestanding Core Utility

The `sleep.elf` executable (`/bin/sleep.elf` and `/bin/sleep`) is a freestanding Ring 3 core utility that pauses execution for a specified amount of time.

---

## 1. Architectural Purpose

As part of Keira Milestone 37 (Text Processing Pipeline Suite), `sleep.elf` introduces deterministic process timing and synchronization:

1. **Ring 3 Privilege Isolation**: Executes entirely in unprivileged userspace, delegating sleep intervals to the kernel through `sys_nanosleep`.
2. **Flexible Time Suffixes**: Supports pure numerical seconds or suffixed notation (`s` for seconds, `m` for minutes, `h` for hours, `d` for days).
3. **Script Coordination**: Enables delay loops, polling intervals, and periodic monitoring in userland shell scripts.

---

## 2. Command-Line Options

| Option | Long Option | Description |
| :--- | :--- | :--- |
| `<number>[smhd]` | *N/A* | Suspend execution for the given duration |
| `-h` | `--help` | Display usage summary and exit with status 0 |

---

## 3. Usage Examples

### Delaying Execution

```bash
# Suspending execution for 1 second:
keira:/# sleep 1

# Suspending execution for 2 seconds using explicit suffix:
keira:/# sleep 2s
```

### Script Timing

```bash
# Periodic check loop in shell:
keira:/# sh -c 'while true; do ping -c 1 10.0.2.2; sleep 5; done'
```

---

## 4. Technical Specifications

- **Source Path**: `userland/bin/sleep/main.c`
- **Binary Locations**: `/bin/sleep.elf`, `/bin/sleep`
- **Privilege Level**: Ring 3 unprivileged
- **Supported Architectures**: `x86_64` (64-bit Long Mode), `i686` (32-bit Protected Mode)
- **Standard Library Dependency**: `libc.a` (freestanding minimal C runtime)
