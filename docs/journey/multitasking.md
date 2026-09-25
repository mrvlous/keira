<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Journey Milestone 3: Preemptive Multitasking & Task Scheduling

This milestone covers the implementation of cooperative and preemptive multitasking, hardware timer ticks, and POSIX process control.

---

## Key Achievements

1. **Round-Robin Scheduler**: 1000 Hz timer tick preemption, state tracking (`Ready`, `Running`, `Blocked`, `Zombie`).
2. **Process Control Block (PCB)**: Managing task IDs, registers, stack pointers, open file descriptors, and virtual memory mappings.
3. **Control Groups (cgroups)**: CPU quota limits, task count throttling, and resource hierarchy isolation.
4. **POSIX Signals**: Signal mask queues, signal delivery frames, and `sigreturn` userland context restoration.
