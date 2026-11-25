.. SPDX-License-Identifier: GPL-2.0-only


Kernel Wait Queues (wait_queue_head_t)
======================================

Keira provides non-allocating kernel wait queues (``WaitQueueHead`` and ``WaitQueueEntry``) inside ``crates/task/src/wait/`` to replace ad-hoc polling and manual ``TaskState::Blocked`` transitions with race-free sleep and wakeup synchronization.

1. Synchronization Model & Exclusivity
======================================

* **Locking Discipline**: Each ``WaitQueueHead`` embeds a dedicated ``IrqSpinLock`` with ``LockRank::Scheduler`` protecting up to ``MAX_WAITERS_PER_QUEUE`` (16) bounded entries.
* **Non-Exclusive vs. Exclusive Waiters**: Waiters tagged with ``WQ_FLAG_EXCLUSIVE`` mitigate thundering herd contention by waking at most one exclusive task per ``wake_up()`` call, while ``wake_up_all()`` broadcasts to all waiting tasks.
* **Condition Wait Macros**: ``wait_event!(wq, cond)`` and ``wait_event_timeout!(wq, cond, timeout_ms)`` prepare the current task, re-verify predicates to guard against lost wakeups and yield cleanly via ``int 32``.

2. Subsystem Wait Queues
========================

The global wait queue registry exposes six kernel synchronization queues:

* ``child_exit_wq``: Parent process blocking in ``sys_waitpid`` and child exit signaling.
* ``pipe_read_wq``: Anonymous pipe reader readiness upon writer buffer insertion or EOF.
* ``pipe_write_wq``: Anonymous pipe writer capacity notifications upon reader consumption.
* ``kswapd_wq``: Background memory page reclamation daemon ``[kswapd0]``.
* ``kflushd_wq``: Background dirty page cache writeback daemon ``[kflushd]``.
* ``workqueue_wq``: Deferred workqueue executor daemon ``[kworker/0:0]``.
