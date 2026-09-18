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

---

## Signal Masking & Pending Queues (`sigprocmask`, `sigpending`)

Processes can dynamically block and unblock signals using `sigprocmask()` (vector 81):
- `SIG_BLOCK` (`0`): Adds the specified signal set to the task's active `signal_mask`.
- `SIG_UNBLOCK` (`1`): Removes the specified signals from `signal_mask` and immediately delivers any queued pending signals.
- `SIG_SETMASK` (`2`): Replaces `signal_mask` with the given set.

Signals generated while masked are recorded in `Task::pending_signals` (queriable via `sigpending()`). `SIGKILL` (`9`) and `SIGSTOP` (`19`) can never be masked or ignored.

---

## Hardware Exception Traps & Fault Containment

Hardware CPU exceptions in user space are mapped directly into standard POSIX signals by the exception dispatcher (`crates/syscall/src/exception/mod.rs`):
- `#UD` (Invalid Opcode, Vector 6) $\rightarrow$ `SIGILL` (4)
- `#DE` (Divide Error, Vector 0) / `#OF` (Overflow, Vector 4) $\rightarrow$ `SIGFPE` (8)
- `#GP` (General Protection, Vector 13) / `#PF` (Page Fault, Vector 14) $\rightarrow$ `SIGSEGV` (11)
- `#NP` (Segment Not Present, Vector 11) / `#SS` (Stack Fault, Vector 12) $\rightarrow$ `SIGBUS` (7)

If the process has not registered a custom signal handler, the task terminates with `TaskState::Zombie(-(sig as i32))`. The kernel serial logger records the fault, and a full register dump is written to `/data/log/core_<pid>.dmp`.

---

## Waitpid Signal Decoding (`sys_waitpid`)

When a parent process inspects a child process terminated by a signal via `waitpid(pid, &wstatus, options)`:
- `wstatus` is encoded as `(-code) & 0x7f`.
- `WIFSIGNALED(wstatus)` evaluates to true.
- `WTERMSIG(wstatus)` returns the terminating POSIX signal number.
