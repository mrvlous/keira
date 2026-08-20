<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Preemptive Task Scheduler

Documentation for scheduling in [`crates/task/src/scheduler.rs`](../../../crates/task/src/scheduler.rs).

## Architecture
- Preemptive round-robin scheduler triggered on 1ms timer ticks (PIT/APIC).
- Supports up to `MAX_TASKS` (64) concurrent kernel threads and userland tasks.
- Context switching preserves CPU registers (RAX, RBX, RCX, RDX, RSI, RDI, RBP, R8-R15, RSP, RIP, RFLAGS, CR3).
- Implements `spawn`, `fork_current_task`, `exit_current`, and `wait_for_task`.
