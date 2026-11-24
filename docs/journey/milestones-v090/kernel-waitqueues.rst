.. SPDX-License-Identifier: GPL-2.0-only


Milestone 67: Linux-Grade Kernel Wait Queues & Sleep Synchronization Engine (wait_queue_head_t)
===============================================================================================

Milestone 67 introduces a Linux-grade kernel wait queue and condition sleep synchronization subsystem (``wait_queue_head_t``) into Keira's Ring 0 task and scheduler core. Expanding upon the binary buddy allocator of Milestone 66 and background worker daemons of Milestone 64, this milestone establishes structured, race-free sleep and wakeup primitives (``prepare_to_wait``, ``finish_wait``, ``wake_up``, ``wake_up_nr``, ``wake_up_all`` and ``wait_event!``), eliminates ad-hoc task state mutation, integrates wait queues into process lifecycle management, IPC anonymous pipes and kernel daemons while introducing dynamic ``/proc/waitqueue`` virtual telemetry alongside a 35th supervisor shell command (``waitqueue``, alias ``wq``).

1. Architectural Motivation & Sleep Synchronization Invariants
==============================================================

In a freestanding Ring 0 monolithic kernel, processes and threads routinely block awaiting external hardware events, child process exits, IPC pipe capacity or kernel timer expirations. Prior to Milestone 67, tasks relied on direct task state assignment (``TaskState::Blocked``) coupled with ad-hoc scheduler dequeueing.

Milestone 67 establishes the following core synchronization invariants:

1. **Structured Event Queues (``WaitQueueHead``)**: Each synchronization point is represented by a dedicated wait queue head containing an interrupt-safe spinlock (``IrqSpinLock``) with ``LockRank::Scheduler`` and a bounded array of waiter entries.
2. **Zero Runtime Dynamic Allocation**: In accordance with Keira's Ring 0 bare-metal safety rules, wait queue registration and wakeup routines operate without dynamic heap allocations. Bounded statically-sized entry tables prevent out-of-memory panics in low-level interrupt handlers and scheduler dispatches.
3. **Thundering Herd Mitigation (Exclusive Wakeups)**: Wait queue entries support the ``WQ_FLAG_EXCLUSIVE`` flag. When waking tasks via ``wake_up()``, the engine wakes all non-exclusive waiters plus at most one exclusive waiter, preventing multiple awakened processes from contending for a single resource.
4. **Condition Loop Guarantees (``wait_event!``)**: To protect against spurious wakeups and race conditions where the condition changes before a task executes, condition loops re-evaluate the target predicate while holding or re-checking the wait queue head state.
5. **Pure Kernel Demarcation**: The wait queue engine operates completely in Ring 0 and is fully operational during pure kernel boot (``make run-bare`` / ``USERLAND=0``).

2. Core Wait Queue Primitives (``crates/task/src/wait/``)
=========================================================

The wait queue subsystem provides canonical Linux-compatible data structures and operations:

* **``WaitQueueEntry``**: Represents a waiting task with its PID, flags (``WQ_FLAG_EXCLUSIVE``, ``WQ_FLAG_WOKEN``) and private context payload.
* **``WaitQueueHead``**: Manages the list of waiting tasks, tracking diagnostic identifiers, lock protection, waiter counts, cumulative wait events, total wakeup signals and monotonic timestamp of the last wakeup.
* **``prepare_to_wait``**: Atomically adds a task entry to the queue, transitions task state to ``TaskState::Blocked`` and dequeues it from the CPU work-stealing runqueue.
* **``finish_wait``**: Removes the task entry from the wait queue and restores its state to ``TaskState::Running`` if it was blocked.
* **``wake_up_nr(nr)``**: Wakes up to ``nr`` exclusive waiters and all non-exclusive waiters, transitioning their states to ``TaskState::Ready`` and enqueuing them onto the local CPU runqueue via ``push_local()``.
* **``wait_event!(wq, condition)``**: Evaluates condition; if false, prepares to wait, yields the CPU (via ``int 32`` interrupt) and finishes wait once condition evaluates to true.
* **``wait_event_timeout!(wq, condition, timeout_ms)``**: Bounded condition sleep with millisecond timeout precision powered by the 8254 PIT / HPET timer.

3. Subsystem Integration
========================

Wait queues are integrated across three primary kernel subsystems:

* **Task Lifecycle (``sys_waitpid`` & ``exit_current``)**: Parent processes waiting for child process termination enqueue on ``CHILD_EXIT_WQ``. When a child terminates via ``exit_current()``, it triggers ``CHILD_EXIT_WQ.wake_up()``, immediately unblocking the parent without polling.
* **IPC Anonymous Pipes (``crates/ipc/src/pipe/fifo.rs``)**: Empty pipes signal reader readiness on ``PIPE_READ_WQ`` when bytes are written by ``write_pipe_channel()``. Full pipes signal writer capacity on ``PIPE_WRITE_WQ`` when bytes are consumed by ``read_pipe_channel()``.
* **Kernel Worker Daemons (``[kswapd0]`` & ``[kflushd]``)**: Background page reclamation and cache writeback daemons register on ``KSWAPD_WQ`` and ``KFLUSHD_WQ``, sleeping in low-power states until signaled by memory watermarks or filesystem sync requests.

4. Dynamic ProcFS Telemetry (``/proc/waitqueue``)
=================================================

Keira exposes real-time wait queue synchronization telemetry via the virtual file ``/proc/waitqueue``:

.. code-block:: text

   # Name                 Waiters  Total Waits      Wakeups    Last Wake (ms)
   child_exit_wq                0            4            4               120
   pipe_read_wq                 0            8            8               342
   pipe_write_wq                0            2            2               338
   kswapd_wq                    1           12           11               450
   kflushd_wq                   1            8            7               410

The ProcFS node dynamically queries the kernel wait queue registry without heap allocation, formatting active waiter counts and historical statistics into human-readable columns.

5. Supervisor Diagnostic Shell Command (``waitqueue``)
======================================================

Milestone 67 expands Keira's Ring 0 diagnostic console from 34 to 35 commands with the addition of ``waitqueue`` (alias ``wq``):

.. code-block:: text

   keira> waitqueue -h
   Usage: waitqueue [-v] [-h]

   Description:
     Display kernel wait queues and condition sleep synchronization.

   Options:
     -v, --verbose Display waiting task PIDs registered on each wait queue
     -h, --help    Show this help message and exit

With the ``-v`` flag, the command displays the exact task PIDs currently sleeping on each queue, enabling instant debugging of deadlocks or stalled worker threads.
