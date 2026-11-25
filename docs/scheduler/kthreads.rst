.. SPDX-License-Identifier: GPL-2.0-only


Ring 0 Kernel Threads & Background Daemons
==========================================

Keira implements native Ring 0 kernel threads (``kthreads``) modeled directly after Linux kernel task infrastructure. Kernel threads execute purely in supervisor mode (Ring 0) using higher-half virtual address mapping and dedicated kernel stack frames without userspace VMAs or memory contexts.

1. Architectural Overview & Linux Parity
----------------------------------------

In a monolithic kernel, deferred asynchronous operations such as page reclamation, dirty buffer synchronization and zombie process harvesting must execute in background execution contexts. Rather than relying on userspace daemons, Keira provisions native in-kernel execution contexts managed by the work-stealing scheduler.

A kernel thread differs from a standard userspace task in several key aspects:

* **Privilege Domain**: Executes permanently in Ring 0 (code segment ``0x08``, data segment ``0x10``).
* **Address Space**: Shares the active higher-half kernel page table hierarchy (PML4).
* **Execution Stack**: Backed by a dedicated page-allocated kernel stack frame.
* **Introspection**: Identified in ``pidinfo`` and userspace ``ps`` with bracketed command names (``[kthreadd]``, ``[kswapd0]``, ``[kflushd]``, ``[kworker/0:0]``) and an empty ``/proc/<pid>/cmdline``.

2. Canonical Kernel Daemons
---------------------------

During scheduler subsystem initialization (``keira_task::scheduler::init()``), Keira initializes four baseline kernel daemons:

1. **``[kthreadd]`` (PID 1)**: The master kernel thread daemon serving as the structural ancestor and manager for deferred kernel worker contexts.
2. **``[kswapd0]`` (PID 2)**: Background physical memory reclamation and slab cache monitoring daemon waiting on ``KSWAPD_WQ``.
3. **``[kflushd]`` (PID 3)**: Background dirty page cache buffer flush and filesystem metadata synchronization daemon waiting on ``KFLUSHD_WQ``.
4. **``[kworker/0:0]`` (PID 4)**: Deferred kernel workqueue executor thread draining ``system_wq`` and ``events_unbound`` upon ``WORKQUEUE_WQ`` wakeups.

3. Kernel Thread Creation API
-----------------------------

Kernel threads are spawned via the ``kthread_create`` primitive defined in ``crates/task/src/scheduler/kthread.rs``:

.. code-block:: rust

   pub fn kthread_create(name: &'static str, entry: fn()) -> Result<usize, &'static str>;

When invoked:

1. The scheduler acquires ``SCHEDULER_LOCK`` and locates a vacant task control block (TCB) slot in ``TASKS``.
2. A dedicated contiguous kernel stack is allocated via the physical memory manager (PMM).
3. The initial ``InterruptContext`` is initialized at the top of the stack, setting the entry point instruction pointer (``RIP``/``EIP``), code segment (``CS = 0x08``), stack segment (``SS = 0x10``) and interrupt flag (``RFLAGS = 0x202``).
4. The task descriptor is marked with ``is_user = false`` and transitioned to ``TaskState::Ready`` or ``TaskState::Blocked``.
5. The task index is returned to the caller.
