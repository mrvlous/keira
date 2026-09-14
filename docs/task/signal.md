<!-- SPDX-License-Identifier: GPL-2.0-only -->

# POSIX Signals & Job Control Tables

This document details asynchronous signal dispatching, signal masks, and background/foreground job control tables in Keira Kernel.

---

## Supported Signals

| Signal Number | Name | Default Action | Overridable |
| :--- | :--- | :--- | :--- |
| `1` | `SIGHUP` | Terminate Process | Yes |
| `2` | `SIGINT` | Terminate Process (Ctrl+C) | Yes |
| `3` | `SIGQUIT` | Terminate Process (Core Dump) | Yes |
| `4` | `SIGILL` | Terminate Process (Illegal Instruction) | Yes |
| `5` | `SIGTRAP` | Trace / Breakpoint Trap | Yes |
| `6` | `SIGABRT` | Abort Signal | Yes |
| `7` | `SIGBUS` | Bus Error (Alignment) | Yes |
| `8` | `SIGFPE` | Floating Point Exception | Yes |
| `9` | `SIGKILL` | Unconditional Immediate Termination | No |
| `10` | `SIGUSR1` | User-Defined Signal 1 | Yes |
| `11` | `SIGSEGV` | Segmentation Violation | Yes |
| `12` | `SIGUSR2` | User-Defined Signal 2 | Yes |
| `13` | `SIGPIPE` | Broken Pipe Write | Yes |
| `14` | `SIGALRM` | Real-Time Timer Alarm | Yes |
| `15` | `SIGTERM` | Graceful Termination Request | Yes |
| `17` | `SIGCHLD` | Child Process State Changed | Yes |
| `18` | `SIGCONT` | Resume Suspended Process | Yes |
| `19` | `SIGSTOP` | Unconditional Process Suspension | No |

---

## Signal Delivery Cycle

Before resuming a task from an interrupt or system call return path, the kernel checks `pending_signals & ~signal_mask`. If a signal is pending:
1. Kernel sets up a signal frame on the userland stack.
2. Changes the return `RIP`/`EIP` to the registered signal handler address.
3. Upon handler completion, userland executes `sys_sigreturn()` to restore original execution context.
