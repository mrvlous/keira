<!-- SPDX-License-Identifier: GPL-2.0-only -->

# `sh.elf` Canonical Ring 3 Userspace Shell

The canonical Ring 3 userspace shell executable (`/bin/sh.elf` and `/bin/sh`) provides a freestanding POSIX-compliant command-line environment running unprivileged in Ring 3 userspace.

---

## 1. Architectural Purpose

Keira establishes a clear architectural boundary between kernel-space supervisory control and unprivileged userspace application execution:

1. **Ring 3 Privilege Isolation**: Runs entirely in unprivileged Ring 3 with memory paging and system call vector protections, preventing userspace crashes from compromising kernel stability.
2. **POSIX Process Control**: Dispatches executable programs using canonical `fork()`, `execve()` and `waitpid()` system calls rather than supervisor-level kernel execution routines.
3. **$PATH Binary Resolution**: Automatically resolves command names without explicit directory paths by searching `/bin/<cmd>` and `/bin/<cmd>.elf` via the POSIX `access()` system call.
4. **Interactive & Non-Interactive Modes**: Provides both an interactive REPL (`sh-0.7$ `) and non-interactive command string execution via the `-c` flag.

---

## 2. Built-in Commands

The userspace shell implements essential built-in commands natively within the binary without spawning sub-processes:

| Command | Usage | Description |
| :--- | :--- | :--- |
| `cd` | `cd [dir]` | Changes current working directory (defaults to `/` if omitted) |
| `pwd` | `pwd` | Displays the current working directory path |
| `echo` | `echo [args...]` | Prints arguments to standard output with `$?` and environment variable expansion |
| `export` | `export [VAR=VAL]` | Sets an environment variable or prints existing environment variables |
| `clear` | `clear` | Clears terminal screen via standard ANSI escape codes (`\x1b[2J\x1b[H`) |
| `help` | `help` | Displays built-in command summary and shell usage guidance |
| `exit` | `exit [code]` | Terminates the shell process with the specified exit status |

---

## 3. Invocation & Execution Examples

### Supervisor Command Bridge (`sh`)

The supervisor console provides a first-class `sh` command that launches `/bin/sh.elf`:

```bash
# Non-interactive command string execution:
keira:/# sh -c echo hello_userspace_shell
hello_userspace_shell

# Running built-ins via -c:
keira:/# sh -c pwd
/

# Spawning child userland binaries:
keira:/# sh -c sysinfo.elf
[SYSINFO] Keira System Telemetry & Hardware Report
...
```

### Direct `$PATH` Execution

Users can invoke `sh` or `sh.elf` directly from the supervisor prompt without typing `run`:

```bash
# Direct binary execution:
keira:/# sh.elf -c help
Keira Userspace Shell (sh)
Built-in commands:
  cd [dir]       - Change current working directory
  pwd            - Print current working directory
  echo [args]    - Print arguments to standard output
  export [k=v]   - Set or list environment variables
  clear          - Clear terminal screen
  help           - Display this help message
  exit [code]    - Exit shell
```

---

## 4. Technical Specifications

- **Source Path**: `userland/bin/sh/main.c`
- **Binary Locations**: `/bin/sh.elf`, `/bin/sh`
- **Privilege Level**: Ring 3 unprivileged
- **Dependencies**: Freestanding `libc.a` (`<stdio.h>`, `<stdlib.h>`, `<string.h>`, `<unistd.h>`, `<sys/wait.h>`)
- **System Calls**: `sys_open`, `sys_close`, `sys_read`, `sys_write`, `sys_fork`, `sys_execve`, `sys_waitpid`, `sys_chdir`, `sys_getcwd` and `sys_exit`
