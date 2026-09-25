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
