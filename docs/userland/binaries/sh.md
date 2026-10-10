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
5. **Pipelines & Redirection**: Streams standard input and output between processes and files using kernel pipes (`sys_pipe`) and descriptor duplication (`sys_dup2`).

---

## 2. Built-in Commands

The userspace shell implements essential built-in commands natively within the binary without spawning sub-processes:

| Command | Usage | Description |
| :--- | :--- | :--- |
| `cd` / `go` | `cd [dir]` | Changes current working directory (defaults to `/` if omitted) |
| `pwd` | `pwd` | Displays the current working directory path |
| `echo` | `echo [args...]` | Prints arguments to standard output with `$?` and environment variable expansion |
| `export` | `export [VAR=VAL]` | Sets an environment variable or prints existing environment variables |
| `clear` | `clear` | Clears terminal screen via standard ANSI escape codes (`\x1b[2J\x1b[H`) |
| `help` | `help` | Displays built-in command summary and shell usage guidance |
| `exit` | `exit [code]` | Terminates the shell process with the specified exit status |

---

## 3. Advanced Shell Execution Semantics

Starting with Milestone 22, the userspace shell supports UNIX stream operators, command pipelines and conditional chaining:

### A. I/O Redirection
- **Output Redirection (`>`)**: Redirects standard output to a file, truncating existing content or creating the file if absent.
  ```bash
  echo "kernel milestone 22" > /tmp/output.txt
  ```
- **Append Redirection (`>>`)**: Appends standard output to the specified file without overwriting existing data.
  ```bash
  echo "additional line" >> /tmp/output.txt
  ```
- **Input Redirection (`<`)**: Streams file contents into standard input of the command.
  ```bash
  cat < /tmp/output.txt
  ```

### B. Pipelines (`|`)
- Connects the standard output of the preceding command to the standard input of the succeeding command via an anonymous kernel pipe.
  ```bash
  echo pipeline_data | cat
  ```

### C. Conditional Command Chaining
- **Logical AND (`&&`)**: Executes the succeeding command only if the preceding command exits successfully (status code 0).
  ```bash
  echo step_one && echo step_two
  ```
- **Logical OR (`||`)**: Executes the succeeding command only if the preceding command fails (non-zero exit code).
  ```bash
  cat /nonexistent || echo "fallback handler"
  ```
- **Sequential Execution (`;`)**: Executes multiple commands in succession regardless of return status.
  ```bash
  pwd ; sysinfo.elf
  ```

---

## 4. Invocation & Execution Examples

### Supervisor Command Bridge (`sh`)

The supervisor console provides a first-class `sh` command that launches `/bin/sh.elf`:

```bash
# Non-interactive command string execution:
keira:/# sh -c echo hello_userspace_shell
hello_userspace_shell

# I/O redirection:
keira:/# sh -c echo m22_success > /tmp/out.txt
keira:/# cat.elf /tmp/out.txt
m22_success

# Conditional chaining:
keira:/# sh -c echo chain_one && echo chain_two
chain_one
chain_two

# Pipelines:
keira:/# sh -c echo streamed_line | cat
streamed_line

# Executing shell scripts:
keira:/# sh /etc/init.sh
[KFS] Initializing userland system environment...
/
[SYSINFO] Keira System Telemetry & Hardware Report
...
[KFS] System initialization sequence completed.
```

### Direct `$PATH` & Shebang Script Execution

Users can invoke `sh` or `sh.elf` directly from the supervisor prompt without typing `run` and execute `.sh` scripts directly via shebang:

```bash
# Direct binary execution:
keira:/# sh.elf -c help
Keira Standalone Userspace Shell (sh v0.7.0)
Built-in Commands:
  cd [dir]       Change current working directory (alias: go)
  pwd            Print working directory
  echo [args]    Display text or environment variables
  export [K=V]   Set environment variable
  clear          Clear console screen
  help           Display this reference manual
  exit [code]    Exit userspace shell

# Direct shebang script execution:
keira:/# demo.sh
Executing script: /bin/demo.sh
[DEMO] Shebang execution active via /bin/sh
[DEMO] Current directory:
/
[DEMO] Demonstration completed successfully.
```

---

## 5. Technical Specifications

- **Source Path**: `userland/bin/sh/main.c`
- **Binary Locations**: `/bin/sh.elf`, `/bin/sh`
- **Privilege Level**: Ring 3 unprivileged
- **Dependencies**: Freestanding `libc.a` (`<stdio.h>`, `<stdlib.h>`, `<string.h>`, `<unistd.h>`, `<sys/wait.h>`)
- **System Calls**: `sys_open`, `sys_close`, `sys_read`, `sys_write`, `sys_pipe`, `sys_dup2`, `sys_fork`, `sys_execve`, `sys_waitpid`, `sys_chdir`, `sys_getcwd` and `sys_exit`
- **Script Engine**: Supports `#!/bin/sh` shebang headers, `#` comment filtering and line-by-line sequential command dispatch
