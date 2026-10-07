<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Context Switching & Register Preservation

Context switching transfers execution between tasks:
1. Save volatile and callee-saved registers onto the current task stack.
2. Update the TSS `RSP0` / `ESP0` pointer with the target task kernel stack.
3. Switch the active address space by reloading register `CR3`.
4. Restore registers and return via `IRETQ` / `IRETD`.

---

## Scheduler Telemetry & Context Switch Accounting

The scheduler dispatch engine (`crates/task/src/scheduler/dispatch/tick.rs`) tracks scheduling metrics:
* `scheduler_get_stats()`: Computes `(total_context_switches, total_ticks, active_tasks)`.
* Every task tracks `cpu_ticks` (time spent running on the CPU) and `switches` (inbound context switches).
* Displayed in userspace via the `tasks` and `cpu` shell commands.

---

## Decentralized Dispatch & Work Balancing

Preemptive timer interrupts call `schedule_tick(current_rsp)`:
1. Current running task saves `current_rsp` and transitions to `TaskState::Ready`.
2. The core queries the decentralized balancer via `pick_next_task(core_id, current_idx)`.
3. If another ready task is selected, the outgoing task is re-enqueued onto the local Chase-Lev runqueue (`push_local`) and the CPU switches to the selected task.
4. If no worker tasks are queued and Core 0 is executing, the scheduler alternates with Task 0 if Task 0 is ready.
5. If no other task is ready, the current task resumes execution with minimal overhead.
