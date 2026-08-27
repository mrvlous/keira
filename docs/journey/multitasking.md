<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Milestone 3: Preemptive Multitasking & Context Switching

This journal entry details the development of the preemptive round-robin scheduler, CPU context switches, and process state machines in Keira Kernel.

---

## Engineering Challenges

1. **Context Switch Atomicity**: Swapping the CPU execution context (stack pointer, instruction pointer, general registers) must happen cleanly without allowing an interrupt to interrupt the context switch itself.
2. **Deadlock Prevention in Safe Rust**: Spinlocks guarding the task runqueue must disable hardware interrupts on acquisition to prevent deadlocks when a timer tick occurs while holding the lock.

---

## Solutions & Design Choices

* **`SpinMutex` and `SpinLock` with CLI**: All kernel locks used in scheduling paths automatically disable interrupts (`cli`) before acquiring the lock and restore the previous interrupt state on drop.
* **Preemptive Timer Ticks**: Configured the PIT timer for a 1000 Hz system tick (1ms quantum) to drive fair round-robin scheduling.
