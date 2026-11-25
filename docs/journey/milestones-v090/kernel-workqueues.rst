.. SPDX-License-Identifier: GPL-2.0-only


Milestone 68: Linux-Grade Kernel Workqueues, Deferred Work Engine & [kworker/0:0]
=================================================================================

Milestone 68 introduces a Linux-grade **Kernel Workqueue & Deferred Execution Engine** (``workqueue_struct``, ``work_struct``, ``delayed_work`` and ``[kworker/0:0]``) into Keira's Ring 0 task and concurrency core. Built directly upon the wait queue subsystem of Milestone 67 and active kernel worker threads of Milestone 64, this milestone allows hardware interrupt handlers, system call dispatchers and kernel maintenance routines to defer work into process context without blocking top-half critical paths or allocating heap memory at runtime.

1. Architectural Motivation & Process-Context Deferral
======================================================

In freestanding monolithic kernels, top-half interrupt handlers and IRQ-spinlock critical paths cannot sleep and must finish rapidly. Heavyweight operations such as cache writebacks, delayed garbage collection and asynchronous diagnostic probing require process-context execution managed by dedicated Ring 0 worker threads:

1. **Zero Heap Allocation in Ring 0**: Each ``WorkQueueStruct`` maintains a bounded slot array (``MAX_WORK_ITEMS_PER_QUEUE = 32``) synchronized via ``IrqSpinLock`` with ``LockRank::Scheduler``.
2. **Reentrant Lock Safety**: Pending work callbacks are dequeued into a fixed stack batch and executed strictly outside the workqueue spinlock, allowing callbacks to safely call ``schedule_work()`` recursively.
3. **Wait Queue Sleeping Integration**: Worker daemon ``[kworker/0:0]`` blocks cooperatively on ``WORKQUEUE_WQ`` via Milestone 67 ``WaitQueueHead`` primitives until woken by new work submissions.
4. **Millisecond Delayed Dispatch**: ``DelayedWork`` pairs work descriptors with monotonic 8254 PIT / HPET target execution deadlines and supports explicit cancellation via ``cancel_system_work()``.
5. **Pure Kernel Demarcation**: All workqueue machinery executes autonomously in Ring 0 and operates cleanly during pure freestanding kernel boots (``make run-bare`` / ``USERLAND=0``).

2. Core Workqueue Primitives
============================

Defined inside ``crates/task/src/workqueue/``, the engine provides standard Linux semantics:

* **``WorkStruct`` & ``DelayedWork``**: Encapsulates unique monotonic work identifiers, subsystem diagnostic names, ``WorkFn`` callback pointers, opaque payloads, state flags (``WORK_STRUCT_PENDING``, ``WORK_STRUCT_DELAYED``, ``WORK_STRUCT_ACTIVE``, ``WORK_STRUCT_COMPLETED``) and target timestamps.
* **``SYSTEM_WQ`` (``system_wq``)**: Primary shared kernel workqueue for immediate callbacks queued via ``schedule_work()``.
* **``SYSTEM_UNBOUND_WQ`` (``events_unbound``)**: Unbound maintenance workqueue for delayed work descriptors scheduled via ``schedule_delayed_work()``.
* **``process_pending_workqueues()`` & ``flush_system_workqueues()``**: Drains expired work items (respecting ``execute_at_ms <= uptime_ms()``) or forces synchronous execution of every pending item.

3. Worker Daemon & Wait Queue Coupling
======================================

Scheduler startup (``init_kthreads()``) spawns **``[kworker/0:0]`` (PID 4)** as a Ring 0 kernel thread parented to ``[kthreadd]`` (PID 1). When idle, ``[kworker/0:0]`` registers on ``WORKQUEUE_WQ`` in ``TaskState::Blocked``. Enqueuing an immediate job calls ``WORKQUEUE_WQ.wake_up()``, transitioning ``[kworker/0:0]`` back onto the work-stealing runqueue for dispatch.

4. Dynamic ProcFS Telemetry
===========================

Virtual file ``/proc/workqueues`` exposes live Ring 0 workqueue telemetry formatted without dynamic allocation:

.. code-block:: text

   # Workqueue       Pending     Queued   Executed  Cancelled  Last Run (ms)
   system_wq               0          4          4          0             85
   events_unbound          1          2          1          0             42

5. Supervisor Console Integration
=================================

Milestone 68 expands Keira's Ring 0 supervisor shell from 35 to 36 diagnostic commands by adding ``workqueue`` (alias ``kworker``):

.. code-block:: text

   keira> workqueue -h
   Usage: workqueue [-v] [-f] [-t] [-h]

   Description:
     Display kernel workqueues and deferred Ring 0 task execution.

   Options:
     -v, --verbose Display queued work item descriptors per workqueue
     -f, --flush   Flush and execute all queued deferred work items
     -t, --test    Schedule a diagnostic probe job and verify execution
     -h, --help    Show this help message and exit
