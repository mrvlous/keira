.. SPDX-License-Identifier: GPL-2.0-only


Kernel Workqueues (workqueue_struct)
====================================

Kernel workqueues in ``crates/task/src/workqueue/`` allow interrupt handlers, system call paths and Ring 0 subsystems to defer heavyweight callbacks into process context executed by ``[kworker/0:0]``.

1. Deferred Work Architecture
=============================

* **Work Descriptors**: ``WorkStruct`` represents immediate deferred callbacks, while ``DelayedWork`` adds millisecond-precision earliest execution timestamps driven by monotonic PIT/HPET ticks.
* **Zero Runtime Allocation**: Each ``WorkQueueStruct`` stores up to ``MAX_WORK_ITEMS_PER_QUEUE`` (32) descriptors inside a bounded fixed array protected by ``IrqSpinLock``.
* **Reentrant Execution Safety**: Pending due callbacks are popped under lock and invoked outside the spinlock so worker functions may safely invoke ``schedule_work()`` recursively without lock acquisition panics.
* **Event-Driven Sleep**: ``schedule_work()`` signals ``WORKQUEUE_WQ.wake_up()``, transitioning ``[kworker/0:0]`` to ``TaskState::Ready`` and enqueueing it on the local SMP runqueue.

2. System Workqueues & Telemetry
================================

Keira maintains two global shared workqueues:

* ``system_wq``: Primary Ring 0 workqueue for immediate asynchronous callbacks scheduled via ``schedule_work()``.
* ``events_unbound``: Unbound maintenance queue supporting delayed jobs scheduled via ``schedule_delayed_work()`` and explicit cancellation via ``cancel_system_work()``.

Live queue counts, pending work items and execution timestamps are exposed via ``/proc/workqueues`` and the ``workqueue`` (alias ``kworker``) supervisor console command.
