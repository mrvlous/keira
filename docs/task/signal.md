<!-- SPDX-License-Identifier: GPL-2.0-only -->

# POSIX Signals & Job Control Tables

This document details asynchronous signal dispatching, signal masks, and background/foreground job control tables in Keira Kernel.

---

## Supported Signals

| Signal Number | Name | Default Action | Overridable |
| :--- | :--- | :--- | :--- |
| `1` | `SIGHUP` | Terminate Process | Yes |
| `2` | `SIGINT` | Terminate Process (Ctrl+C) | Yes |
| `9` | `SIGKILL` | Unconditional Immediate Termination | No |
| `15` | `SIGTERM` | Graceful Termination Request | Yes |
| `19` | `SIGSTOP` | Unconditional Process Suspension | No |
| `18` | `SIGCONT` | Resume Suspended Process | Yes |

---

## Signal Delivery Cycle

Before resuming a task from an interrupt or system call return path, the kernel checks `pending_signals & ~signal_mask`. If a signal is pending:
1. Kernel sets up a signal frame on the userland stack.
2. Changes the return `RIP`/`EIP` to the registered signal handler address.
3. Upon handler completion, userland executes `sys_sigreturn()` to restore original execution context.
