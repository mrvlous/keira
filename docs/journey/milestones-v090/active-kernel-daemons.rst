.. SPDX-License-Identifier: GPL-2.0-only


Milestone 64: Active Ring 0 Background Daemons ([kswapd0] & [kflushd])
======================================================================

Milestone 64 advances Keira's Ring 0 pure kernel architecture by activating background kernel worker daemons (``kthreads``) into fully autonomous, event-driven and timer-assisted subsystems. Following the introduction of daemon process descriptors in Milestone 62 and dynamic ELF modules in Milestone 63, this milestone implements active background memory reclamation via ``[kswapd0]`` and dirty page cache writeback synchronization via ``[kflushd]``.

1. Architectural Motivation & Pure Ring 0 Kernel Threads
========================================================

In monolithic kernels modeled after Linux, background asynchronous maintenance tasks must execute within dedicated kernel contexts rather than blocking foreground user applications or shell consoles:

1. **Pure Ring 0 Execution**: Kernel threads (``kthreads``) have no user-mode address space, no Ring 3 stacks and no userspace C runtime (``libc``). They execute strictly in supervisor mode and are scheduled by the kernel scheduler alongside other tasks.
2. **Page Cache Dirty Writeback (``[kflushd]``)**: When tasks write file data through the Virtual Filesystem (VFS), data is cached in 4096-byte page cache buffer heads and flagged as dirty. An autonomous daemon must periodically audit the page cache, lock dirty entries, commit modified blocks to physical storage (SATA AHCI / NVMe / RAM disk) and clear dirty flags.
3. **Asynchronous Memory Reclamation (``[kswapd0]``)**: When physical memory allocation approaches high exhaustion, allocating threads should not stall waiting for synchronous frame compaction. An asynchronous daemon continuously audits memory pressure against low and critical watermarks, evicting clean, unreferenced cache pages and staging inactive anonymous memory to swap backing stores.
4. **Kernel Introspection & Telemetry**: Full observability into daemon operations, including scan iterations, writeback passes, dirty page counts and memory pressure events, accessible via supervisor commands (``daemons``) and virtual pseudo-filesystems (``/proc/daemons``).

2. Dirty Page Cache Writeback Engine (``crates/fs/src/cache/page.rs``)
======================================================================

The unified page cache engine provides background dirty tracking and safe writeback dispatch:

* **Dirty Page Accounting**: Tracks uncommitted modifications using ``PAGE_FLAG_DIRTY``. The helper ``count_dirty_pages()`` computes the resident dirty working set across the 32-slot (128 KiB) unified cache pool.
* **Lock-Protected Writeback**: The ``kflushd_step()`` routine audits the cache. If dirty entries exist, it locks each page with ``PAGE_FLAG_LOCKED``, invokes the underlying block device commit callback, clears the dirty flag and releases the lock.
* **Clean Page Eviction**: The ``evict_clean_pages(max_pages)`` interface enables the memory reclamation daemon (``[kswapd0]``) to safely evict valid, non-dirty and unlocked page frames, reclaiming physical memory without data loss.
* **Writeback Telemetry**: Tracks cumulative writeback passes (``KFLUSHD_PASSES``), pages committed (``KFLUSHD_PAGES_COMMITTED``) and monotonic timestamps (``KFLUSHD_LAST_TICKS``) exposed via ``get_kflushd_telemetry()``.

3. Physical Memory Reclamation Engine (``crates/mem/src/vmm/reclaim.rs``)
=========================================================================

The virtual memory subsystem integrates watermark-driven asynchronous reclamation:

* **Three-Tier Watermark Model**:
  * **Normal (> 20% free RAM)**: Memory is abundant; ``[kswapd0]`` remains in cooperative sleep.
  * **Low (< 20% free RAM)**: Memory pressure is detected; ``[kswapd0]`` triggers moderate reclamation sweeps (targeting up to 4 clean pages per pass).
  * **Critical (< 5% free RAM)**: Severe memory starvation; ``[kswapd0]`` triggers urgent reclamation sweeps (targeting up to 16 clean pages per pass).
* **Decoupled Reclamation Traversal**: The ``kswapd_step(reclaim_fn)`` interface accepts an optional reclamation function pointer, allowing ``keira-task`` to connect ``keira-fs`` page cache eviction without introducing circular crate dependencies.
* **Reclamation Telemetry**: Records memory audit sweeps (``KSWAPD_SCANS``), reclaimed physical frames (``KSWAPD_PAGES_RECLAIMED``) and pressure threshold crossings (``KSWAPD_PRESSURE_EVENTS``) exposed via ``get_kswapd_telemetry()``.

4. Supervisor Console & Telemetry Commands (``crates/shell/src/cmds/sys.rs``)
=============================================================================

Milestone 64 introduces the ``daemons`` command to the supervisor emergency console:

* **Syntax**: ``daemons [-h] [-f] [-r]`` (alias: ``kthreads``).
* **Options**:
  * ``-f, --flush``: Forces an immediate manual writeback pass on ``[kflushd]``, committing all resident dirty pages to storage.
  * ``-r, --reclaim``: Forces an immediate manual memory reclamation pass on ``[kswapd0]``, evicting clean cache pages.
  * ``-h, --help``: Displays command usage and reference options.
* **Status Output**: Formats real-time metrics for ``[kthreadd]`` (PID 1), ``[kswapd0]`` (PID 2) and ``[kflushd]`` (PID 3), displaying watermark levels, current memory pressure state, dirty page counts and cumulative writebacks.

5. ProcFS Pseudo-Filesystem Telemetry (``crates/fs/src/proc/system/daemons.rs``)
================================================================================

The kernel virtual filesystem provides structured daemon telemetry via ``/proc/daemons``:

.. code-block:: text

   kthreadd: pid=1 state=active role=coordinator
   kswapd0: pid=2 scans=42 reclaimed=12 pressure_events=1 pressure=normal
   kflushd: pid=3 passes=50 flushed=8 dirty_pages=0

This file is fully integrated into ``PROC_ENTRIES``, enabling userspace utilities, shell scripts and supervisor tools to query kernel daemon telemetry dynamically.
