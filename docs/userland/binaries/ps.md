<!-- SPDX-License-Identifier: GPL-2.0-only -->

# `ps.elf` Freestanding Core Utility

The `ps.elf` executable (`/bin/ps.elf` and `/bin/ps`) is a freestanding Ring 3 core utility that reports a snapshot of current active processes.

---

## 1. Architectural Purpose

In accordance with pure kernel design and UNIX standards, process enumeration is completely decoupled from Ring 0 supervisor space:

1. **ProcFS Dynamic Querying**: Inspects active processes through the virtual filesystem by reading `/proc/<pid>/status` for PID 0 through 64.
2. **Tabular Formatting**: Extracts process metadata including Process Name, State (`R` for Running, `S` for Sleeping, `Z` for Zombie, `D` for Blocked), Process ID (PID) and Parent Process ID (PPID).
3. **Unprivileged Execution**: Runs in Ring 3 userspace without requiring supervisor privileges or in-kernel private hooks.

---

## 2. Command-Line Options

| Option | Long Option | Description |
| :--- | :--- | :--- |
| `-h` | `--help` | Displays usage summary and exits with status 0 |

---

## 3. Usage Examples

```bash
# Display active process table:
keira:/# ps
  PID  PPID S COMMAND
    0     0 S idle
    1     0 S init.elf
    2     1 R ps.elf

# Running via canonical /bin/ps alias:
keira:/# /bin/ps
  PID  PPID S COMMAND
    0     0 S idle
    1     0 S init.elf
    2     1 R ps
```

---

## 4. Technical Specifications

- **Source Path**: `userland/bin/ps/main.c`
- **Binary Locations**: `/bin/ps.elf`, `/bin/ps`
- **Privilege Level**: Ring 3 unprivileged
- **Dependencies**: Freestanding `libc.a`, `sys_open`, `sys_read`, `sys_close`, `sys_write`
