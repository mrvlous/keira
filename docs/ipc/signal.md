<!--
SPDX-License-Identifier: GPL-2.0-only

Keira Kernel - Operating System Kernel
Copyright (C) 2026 Moh. Ananda Firmansyah Putra
-->

# POSIX Real-Time Signal Engine & Process Job Control Subsystem

This document details POSIX signal dispatching, signal masks, Ring 3 stack frame contexts, and terminal job control state management in Keira Kernel.

## 1. POSIX Signal Engine Architecture
The signal subsystem ([signal.rs](../../kernel/src/sched/signal.rs)) provides asynchronous inter-process notification and termination (**Syscall 72: `sys_kill`**).

### Supported POSIX Signal Vectors
*   **`SIGHUP` (1)**: Hangup detected on controlling terminal.
*   **`SIGINT` (2)**: Terminal interrupt signal (`Ctrl+C`).
*   **`SIGQUIT` (3)**: Terminal quit signal (`Ctrl+\`).
*   **`SIGKILL` (9)**: Immediate process force termination (uncatchable).
*   **`SIGSEGV` (11)**: Invalid virtual memory segment violation.
*   **`SIGALRM` (14)**: Real-time timer alarm expiration.
*   **`SIGTERM` (15)**: Graceful process termination request.
*   **`SIGCHLD` (17)**: Child process state change or exit status notification.
*   **`SIGCONT` (18)**: Resume execution of stopped process.
*   **`SIGSTOP` (19)**: Stop process execution.

---

## 2. Terminal Job Control Table (`JOB_TABLE`)
Keira Kernel shell tracks active background and foreground processes using a 16-slot Job Control Table:

```rust
pub struct JobInfo {
    pub job_id: u32,
    pub pid: u32,
    pub name: [u8; 32],
    pub name_len: usize,
    pub state: JobState,
    pub is_foreground: bool,
}
```

---

## 3. Shell Commands
*   **`kill [-signal_number] <pid>`**: Dispatches POSIX signal to target process PID via Syscall 72.
*   **`jobs`**: Displays active background and stopped process jobs.
*   **`fg <job_id>`**: Brings target background job to terminal foreground context.
*   **`bg <job_id>`**: Resumes execution of stopped process job in background.
