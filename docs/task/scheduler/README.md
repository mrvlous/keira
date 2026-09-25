<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Preemptive Scheduler Architecture

Keira implements an austere, robust preemptive scheduler (`crates/task/src/scheduler/`).

---

## Scheduler Index

| Document | Description |
| :--- | :--- |
| [`core.md`](core.md) | Round-robin queue, 1000 Hz timer preemption, task states |
| [`lifecycle.md`](lifecycle.md) | Task creation (`fork`, `spawn`), state transitions, termination (`exit`, `waitpid`) |
| [`dispatch.md`](dispatch.md) | Context switch routine, register swapping, CR3 page directory switch |
