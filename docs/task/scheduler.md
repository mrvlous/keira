<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Preemptive Multitasking Scheduler

This document details the preemptive round-robin scheduler, timer tick preemption, and context switching mechanics in Keira Kernel.

---

## Scheduling Loop & Preemption

1. **Timer Interrupt**: Every 1ms (1000 Hz PIT tick), the CPU interrupt handler invokes `scheduler_tick()`.
2. **Quantum Consumption**: The active task's remaining time quantum is decremented.
3. **Context Switch**: If the quantum expires, the scheduler saves current registers into the active `TaskContext`, chooses the next `Ready` task from the circular runqueue, and performs a low-level register swap (`switch_context`).

---

## System V User Stack ABI & CLI Arguments

When executing a Ring 3 ELF binary via `sys_exec` or the `run` command, the kernel formats the top of the user stack according to the standard System V ABI:

```
Higher Addresses
+-------------------------------------------------+
| Raw Null-Terminated Argument Strings ("calc.c") |
| ...                                             |
+-------------------------------------------------+
| NULL Pointer (End of envp)                      |
| NULL Pointer (End of argv)                      |
| char *argv[argc - 1]                            |
| ...                                             |
| char *argv[0]                                   |
| uint64_t argc                                   | <-- %rsp (16-byte aligned)
+-------------------------------------------------+
Lower Addresses
```

- **x86_64 Register Calling Convention**: `jump_to_user` forwards `rdi = argc = [rsp]` and `rsi = argv = rsp + 8` into `_start(int argc, char **argv)`.
- **i686 Stack Calling Convention**: Arguments are pushed directly onto the stack and read at `[esp+4]` (`argc`) and `[esp+8]` (`argv`).

---

## POSIX Asynchronous Signal Trampoline & Interactive TTY Signals

Keira supports standard POSIX signal handling:

- `sys_sigaction(pid, signum, handler, old_handler)`: Registers userland signal handlers for signals such as `SIGHUP`, `SIGINT`, `SIGTERM`, `SIGKILL`, `SIGSTOP`, `SIGCONT`.
- `sys_kill(pid, signum)`: Dispatches asynchronous signals to target processes with status transitions (`Running`, `Stopped`, `Terminated`) and wake-up notifications.
- `sys_sigreturn()`: Restores saved user register state upon completing signal handler execution.
- **Interactive TTY Signals**: Keyboard scan codes are processed by the TTY line discipline:
  - `Ctrl+C` (ASCII 3): Automatically routes `SIGINT` (2) to the active foreground process registered in `JOB_TABLE`.
  - `Ctrl+Z` (ASCII 26): Automatically routes `SIGSTOP` (19) to freeze the foreground process and return control to the shell prompt.

---

## Core API (`crates/task/src/scheduler/mod.rs`)

```rust
pub fn init_scheduler();
pub fn schedule();
pub fn spawn_task(entry: usize, is_user: bool, name: &str) -> Result<u32, &'static str>;
pub fn exit_current_task(exit_code: i32) -> !;
pub fn get_current_pid() -> u32;
pub fn fork_current_task() -> Result<usize, &'static str>;
pub fn sys_waitpid(target_pid: i64, status_ptr: *mut i32, options: u32) -> Result<usize, &'static str>;
pub fn send_signal(pid: usize, sig: u32) -> Result<(), &'static str>;
```
