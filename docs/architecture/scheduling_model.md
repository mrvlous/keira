<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Preemptive Scheduling & Task Switching

Keira Kernel implements a tick-driven preemptive round-robin scheduler.

## Timer Interrupt Dispatch
1. 8253 PIT or Local APIC fires an interrupt at 1000Hz (every 1ms).
2. The interrupt handler saves the current CPU register state into an `InterruptContext`.
3. Calls `keira_task::scheduler::schedule_tick()`.
4. Selects the next runnable task from `TASKS[0..MAX_TASKS]`.
5. If the address space changes, writes the new task's PML4 address to `CR3`.
6. Restores the register frame and executes `iretq`.
