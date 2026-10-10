<!-- SPDX-License-Identifier: GPL-2.0-only -->

# `kill.elf` Freestanding Core Utility

The `kill.elf` executable (`/bin/kill.elf` and `/bin/kill`) is a freestanding Ring 3 core utility that sends POSIX signals to specified processes.

---

## 1. Architectural Purpose

Signal transmission is delegated to userspace, utilizing the standard `sys_kill(pid, sig)` system call vector:

1. **POSIX Signal Delivery**: Dispatches signals to target process IDs via vector 18 (`sys_kill`).
2. **Signal Number and Name Resolution**: Resolves numeric signals (e.g. `9`, `15`) and textual signal identifiers (e.g. `SIGTERM`, `SIGKILL`, `SIGINT`, `SIGHUP`).
3. **Signal Table Listing**: Supports `-l`/`--list` to inspect supported signals and their associated numeric values.

---

## 2. Command-Line Options

| Option | Long Option | Description |
| :--- | :--- | :--- |
| `-<sig>` | *N/A* | Sends signal `<sig>` (numeric or name like `-9` or `-TERM`) |
| `-s <sig>` | `--signal <sig>` | Explicitly specifies signal number or signal name |
| `-l` | `--list` | Lists all supported signal names and values |
| `-h` | `--help` | Displays usage summary and exits with status 0 |

---

## 3. Usage Examples

```bash
# Terminate process 42 gracefully (SIGTERM 15):
keira:/# kill 42

# Force kill process 42 (SIGKILL 9):
keira:/# kill -9 42
keira:/# kill -KILL 42

# List all available signals:
keira:/# kill -l
 1) SIGHUP       2) SIGINT       3) SIGQUIT      9) SIGKILL
10) SIGUSR1     11) SIGSEGV     12) SIGUSR2     13) SIGPIPE
14) SIGALRM     15) SIGTERM
```

---

## 4. Technical Specifications

- **Source Path**: `userland/bin/kill/main.c`
- **Binary Locations**: `/bin/kill.elf`, `/bin/kill`
- **Privilege Level**: Ring 3 unprivileged
- **System Call**: `sys_kill` (vector 18)
