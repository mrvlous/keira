<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Milestone 20: Standalone Userspace Shell (`/bin/sh.elf`) & Direct `$PATH` Execution

## Overview

Milestone 20 advances the Keira operating system architecture along the **Road to v0.7.0** by delivering a canonical standalone Ring 3 userspace shell binary (`userland/bin/sh/main.c`) and establishing direct binary execution semantics across the entire operating system.

In earlier milestones, running userland ELF executables required prefixing commands with `run` (e.g. `run /bin/sysinfo.elf`). With Milestone 20, Keira adopts Linux-grade execution semantics:
1. **Direct `$PATH` Execution**: Users can type binary names directly (e.g. `sysinfo`, `sysinfo.elf`, `/bin/sysinfo.elf`, `sh -c "echo hello"`) along with arbitrary command-line arguments. The kernel command router automatically resolves binary paths against `/bin` and the root filesystem, launches them in unprivileged Ring 3 and forwards argument tokens seamlessly.
2. **Canonical Standalone Userspace Shell (`/bin/sh.elf`)**: A pure userspace POSIX-compliant shell running as PID 2 in Ring 3 unprivileged mode, providing interactive and script execution capabilities independent of the supervisor control plane.
3. **Supervisor Bridge Command (`sh`)**: The supervisor shell gains a first-class `sh` command (`crates/shell/src/cmds/sys/control/sh.rs`) that transitions execution directly into `/bin/sh.elf`.

---

## Architectural Motivation: Linux Parity in Userspace

In traditional UNIX and Linux systems, the kernel initializes hardware and mounts root filesystems before transferring execution to userspace `init` (PID 1). Userspace `init` spawns a login manager or userspace shell (such as `/bin/sh` or `/bin/bash`). Command execution within the shell relies on standard `fork()`, `execve()` and `waitpid()` system calls rather than supervisor-level kernel execution routines.

Before Milestone 20, Keira's interactive interface operated exclusively within the supervisor shell control plane. While this provided robust diagnostic access, true operating system compliance required separating the kernel control plane from an unprivileged userspace shell and enabling users to run applications without special kernel invocation syntax.

Milestone 20 resolves this architectural challenge through a dual-layer approach:

```
+-------------------------------------------------------------------------+
|                        SUPERVISOR CONTROL PLANE                         |
|                     (Root Shell: keira:/#, Ring 0)                      |
|                                                                         |
|   Direct Dispatch: [sysinfo.elf] / [kcc /tmp/main.c] / [sh -c "..."]    |
|   Fallback Resolver: run_direct_with_parts() -> $PATH Search (/bin)    |
+------------------------------------+------------------------------------+
                                     |
                                     | jump_to_user()
                                     v
+-------------------------------------------------------------------------+
|                        CANONICAL USERSPACE (RING 3)                      |
|                                                                         |
|   +-----------------------------------------------------------------+   |
|   |                  /bin/sh.elf (POSIX Shell, PID 2)               |   |
|   |  - Prompt: sh-0.7$                                              |   |
|   |  - Built-ins: cd, pwd, echo, export, clear, help, exit          |   |
|   |  - Process Control: fork(), execve(), waitpid()                 |   |
|   |  - $PATH Resolver: /bin/<cmd> and /bin/<cmd>.elf               |   |
|   +-----------------------------------------------------------------+   |
|                                    |                                    |
|             +----------------------+----------------------+             |
|             |                      |                      |             |
|             v                      v                      v             |
|       /bin/sysinfo.elf      /bin/kcc.elf           /bin/init.elf        |
+-------------------------------------------------------------------------+
```

---

## Technical Implementation

### 1. Standalone Userspace Shell (`userland/bin/sh/main.c`)

The userspace shell is written in freestanding C and links against the kernel's freestanding C standard library (`libc.a`).

Key features include:
- **Interactive REPL**: Displays the `sh-0.7$ ` prompt, reads lines via `fgets(line_buf, sizeof(line_buf), stdin)`, parses whitespace-delimited tokens and executes commands.
- **Non-Interactive Execution (`-c`)**: Supports executing one-off command strings, concatenating all subsequent argument tokens (e.g. `sh -c echo hello_userspace_shell`).
- **POSIX Built-in Commands**:
  - `cd [dir]`: Changes current working directory via `chdir()`. Defaults to `/` if omitted.
  - `pwd`: Prints the current working directory via `getcwd()`.
  - `echo [args]`: Prints arguments with support for `$?` (exit status) and environment variable expansion.
  - `export [KEY=VALUE]`: Sets environment variables via `putenv()` or displays current variables.
  - `clear`: Emits standard ANSI clear screen escape sequence `\x1b[2J\x1b[H`.
  - `help`: Displays built-in command reference and usage instructions.
  - `exit [code]`: Terminates the shell process with the requested exit code.
- **Binary Resolution & Execution**: Commands that are not built-ins are resolved against `/bin/<cmd>` and `/bin/<cmd>.elf`. The shell forks a child process with `fork()`, invokes `execve()` and awaits completion via `waitpid()`.

### 2. Standard C Library Enhancements (`userland/include/unistd.h` and `userland/lib/unistd/fs/fs.c`)

To support binary resolution in the userspace shell, the `access()` POSIX system call wrapper was declared in `userland/include/unistd.h` and implemented in `userland/lib/unistd/fs/fs.c`:

```c
int access(const char *pathname, int mode) {
    (void)mode;
    if (!pathname) {
        errno = EINVAL;
        return -1;
    }
    int fd = open(pathname, O_RDONLY, 0);
    if (fd < 0) {
        return -1;
    }
    close(fd);
    return 0;
}
```

### 3. Direct `$PATH` Execution in Supervisor Shell (`crates/shell/src/cmds/proc/task/run.rs`)

To enable Linux-style command execution without typing `run`, the shell executor router was enhanced with `run_direct_with_parts`:

```rust
pub fn run_direct_with_parts(command: &str, parts: &mut core::str::SplitWhitespace) -> bool {
    let mut args_buf: [&str; 16] = [""; 16];
    let mut arg_count = 0;

    args_buf[0] = command;
    arg_count += 1;

    for part in parts.by_ref() {
        if arg_count < 16 {
            args_buf[arg_count] = part;
            arg_count += 1;
        }
    }

    run_direct_with_args(command, &args_buf[..arg_count])
}
```

In `crates/shell/src/executor/dispatch/router.rs`, if a command does not match any registered supervisor command keyword, the router automatically attempts resolution against `/bin` and the root filesystem before reporting an unknown command error.

### 4. Supervisor `sh` Command (`crates/shell/src/cmds/sys/control/sh.rs`)

A first-class `sh` command was integrated into the system control group (`sys`), bringing the total shell command count to **78 commands** (26 `sys` commands).

```
keira:/# sh --help
Usage: sh [-c command] [-h|--help]

Description:
  Launch or execute commands inside the canonical Ring 3 userspace shell.

Options:
  -c <cmd>       Execute command string non-interactively and exit
  -h, --help     Show this help message and exit
```

---

## Verification & Stability Matrix

Every subsystem across both `x86_64` and `i686` targets was tested to **100% zero errors and zero warnings**:

| Verification Target | Command / Harness | Result | Notes |
| :--- | :--- | :--- | :--- |
| **Workspace Unit Tests** | `cargo test --workspace` | `PASS (100%)` | 55 shell tests, 24 syscall tests and 27 task tests passed |
| **Rust Linter** | `cargo clippy --workspace` | `PASS (0 warn)` | Clean dev and release builds |
| **Code Formatting** | `make format` | `PASS` | All Rust and C sources formatted to style guide |
| **C Static Analysis** | `make lint` (`clang-tidy`) | `PASS (0 err)` | Clean static analysis across all userland sources |
| **Dual-Arch Build** | `make full` | `PASS` | `x86_64` and `i686` ISO and FAT16 disk images compiled |
| **Userland Binaries** | `test_userland_binaries.py` | `PASS` | `init.elf`, `sh.elf`, `sysinfo.elf`, `test_threads.elf`, `kcc.elf` verified |
| **Direct Binary Invocation** | QEMU Live Shell | `PASS` | Running `sysinfo.elf` directly without `run` verified |
| **KCC Compilation & Run** | `test_kcc_single.py` | `PASS` | KCC native compiler generated working Ring 3 ELF binary |
| **Live Network APIs** | `test_multi_api.py` | `PASS` | IP geolocation, HTTP echo and download tested |
| **Dual-Arch Command Battery** | `test_dual_arch_all_cmds.py` | `PASS` | 61 commands executed on both architectures |

---

## Conclusion & Next Steps

Milestone 20 bridges the final conceptual gap between kernel supervisor control and POSIX userspace execution. With `/bin/sh.elf` operational in Ring 3 and direct `$PATH` execution enabled across the supervisor console, Keira delivers authentic UNIX development ergonomics while maintaining absolute kernel isolation.
