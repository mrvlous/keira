<!-- SPDX-License-Identifier: GPL-2.0-only -->

# `su.elf` Freestanding Switch User Utility

The `su.elf` executable (`/bin/su.elf` and `/bin/su`) is a freestanding Ring 3 utility that switches the user ID and group ID of the process session via the kernel credentials engine.

---

## 1. Architectural Purpose

In Keira's monolithic design:

1. **Kernel Privilege Demotion & Boundaries**: Calls `setgid()` (Vector 46) and `setuid()` (Vector 23) in Ring 0. Only processes with effective UID 0 are permitted to arbitrarily alter their credentials; unprivileged processes attempting elevation are rejected by the kernel with `EPERM` (`Operation not permitted`).
2. **Session Shell Dispatch**: Configures standard environment variables (`USER`, `LOGNAME`, `SHELL`) and dispatches `/bin/sh` via `execve()` (Vector 11).
3. **Child Lifecycle & Wait**: Because Keira's `execve` spawns an isolated user task and returns the child PID to the parent process, `su` awaits child process completion via `waitpid()`, exiting with the child's exact exit status.

---

## 2. Command-Line Options

| Option | Long Option | Description |
| :--- | :--- | :--- |
| `-c COMMAND` | `--command` | Pass COMMAND to the invoked shell |
| `-s SHELL` | `--shell` | Run specified SHELL (default: `/bin/sh`) |
| `-`, `-l` | `--login` | Simulated login environment |
| `-h` | `--help` | Displays usage reference and exits with status 0 |
| `-v` | `--version` | Displays version information and exits with status 0 |

---

## 3. Usage Examples

```bash
# Switch to unprivileged UID 1000:
keira:/# su 1000
keira:/$ whoami
1000

# Attempt unauthorized privilege escalation (fails with EPERM):
keira:/$ su 0
su: cannot set GID 0: Operation not permitted

# Exit child shell back to parent session:
keira:/$ exit
keira:/# whoami
root
```

---

## 4. Technical Specifications

- **Source Path**: `userland/bin/su/main.c`
- **Binary Locations**: `/bin/su.elf`, `/bin/su`
- **System Call Invocations**: `SYS_SETGID` (Vector 46), `SYS_SETUID` (Vector 23), `SYS_EXECVE` (Vector 11), `SYS_WAITPID` (Vector 7)
- **Privilege Level**: Superuser (UID 0) required for arbitrary switching; demoted execution once switched
