<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Kernel Runtime & Executive Subsystems

This module details runtime executive services, module loading, virtualization, and fault handling.

---

## Runtime Index

| Document | Focus Area | Description |
| :--- | :--- | :--- |
| [`main_loop.md`](main_loop.md) | Executive Loop | Idle task, event polling, and scheduler handoff |
| [`panic.md`](panic.md) | Fault Handling | Panic screen, register dump, stack unwinding |
| [`lkm.md`](lkm.md) | Module Loading | Loadable Kernel Module (LKM) dynamic linking |
| [`kvm.md`](kvm.md) | Virtualization | In-kernel hardware-assisted virtualization hooks |
| [`concurrency.md`](concurrency.md) | Synchronization | Spinlocks, IRQ locks, per-CPU structures |
