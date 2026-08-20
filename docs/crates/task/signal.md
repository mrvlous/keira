<!-- SPDX-License-Identifier: GPL-2.0-only -->

# POSIX Signal Delivery & Job Control

Documentation for signals in [`crates/task/src/signal.rs`](../../../crates/task/src/signal.rs).

## Features
- Dispatches standard POSIX signals: `SIGHUP`, `SIGINT`, `SIGQUIT`, `SIGKILL`, `SIGSEGV`, `SIGALRM`, `SIGTERM`, `SIGSTOP`, `SIGCONT`.
- System call: `sys_kill` (Syscall 72).
- Maintains a 16-slot Terminal Job Control Table (`JOB_TABLE`) for background (`bg`) and foreground (`fg`) process resumption and status tracking.
