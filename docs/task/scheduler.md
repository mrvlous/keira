<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Preemptive Multitasking Scheduler

This document details the preemptive round-robin scheduler, timer tick preemption, and context switching mechanics in Keira Kernel.

---

## Scheduling Loop & Preemption

1. **Timer Interrupt**: Every 1ms (1000 Hz PIT tick), the CPU interrupt handler invokes `scheduler_tick()`.
2. **Quantum Consumption**: The active task's remaining time quantum is decremented.
3. **Context Switch**: If the quantum expires, the scheduler saves current registers into the active `TaskContext`, chooses the next `Ready` task from the circular runqueue, and performs a low-level register swap (`switch_context`).

---

## Core API (`crates/task/src/scheduler/mod.rs`)

```rust
pub fn init_scheduler();
pub fn schedule();
pub fn spawn_task(entry: usize, is_user: bool, name: &str) -> Result<u32, &'static str>;
pub fn exit_current_task(exit_code: i32) -> !;
pub fn get_current_pid() -> u32;
```
