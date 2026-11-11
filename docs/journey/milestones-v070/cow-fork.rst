.. SPDX-License-Identifier: GPL-2.0-only

========================================================================================
Milestone 49: Copy-on-Write (CoW) Fork Engine & Virtual Memory Fault Paging Architecture
========================================================================================

1. Executive Summary
====================

Milestone 49 transforms Keira Kernel's Copy-on-Write (CoW) memory architecture into an industrial-grade, high-concurrency virtual memory paging subsystem. Prior to this milestone, physical frame reference counting relied on an :math:`O(N)` linear array scan across a static 4,096-entry table, causing significant lock contention, high latency under heavy process forking and a catastrophic failure mode where exceeding 4,096 frames resulted in silent memory sharing without tracking.

Through Milestone 49, Keira introduces:

1. **:math:`O(1)` Open-Addressing Hash Table Physical Frame Refcounting Subsystem** expanding tracking capacity to 16,384 frames (:math:`2^{14}` power of 2) with multiplicative Fibonacci hashing ``(frame >> 12).wrapping_mul(0x9E3779B97F4A7C15)``.
2. **Deterministic Knuth Backward-Shift Deletion** preserving linear probing search invariants without tombstones, preventing search path degradation over millions of cycles.
3. **Granular CoW Telemetry Engine** tracking active shared frames, peak shared frames, in-place zero-allocation promotions and physical frame replications.
4. **VMM Unmapping Safety Hardening** replacing raw frame deallocation with reference-counted release in ``free_and_unmap_page``, eliminating premature free bugs during ``munmap``.
5. **SMP TLB Shootdown Synchronization** issuing broadcast invalidations across secondary processor cores during address space cloning and page fault promotions.
6. **Supervisor and ProcFS Integration** exposing live CoW telemetry counters to supervisor commands (``memory -c``, ``memory -p``) and ``/proc/vmstat``.

2. Architecture & Design
========================

.. code-block:: text

   flowchart TD
       subgraph ForkPath["Process Fork Path (clone_user_address_space_cow)"]
           P_PTE["Parent PTE: Set PAGE_COW, Clear PAGE_WRITABLE"] --> RETAIN["pmm::retain_frame(phys_frame)"]
           RETAIN --> HASH["O(1) Hash Table Lookup / Insert"]
           HASH --> C_PTE["Child PTE: Map with PAGE_COW"]
           C_PTE --> TLB_FLUSH["invlpg(vaddr) & tlb_shootdown(0)"]
       end

       subgraph FaultPath["Page Fault Handler (#PF Vector 14)"]
           PF["Page Fault (Present + Write)"] --> CHECK["Is PTE PAGE_COW?"]
           CHECK -->|Yes| REF["pmm::frame_refcount(phys_frame)"]
           REF -->|Refcount <= 1| PROMOTE["In-Place Promotion: Clear PAGE_COW, Set PAGE_WRITABLE"]
           REF -->|Refcount > 1| REPLICATE["Allocate New Frame, Copy 4 KiB, Map Writable"]
           PROMOTE --> INC_PROM["Increment COW_IN_PLACE_PROMOTIONS (Zero Alloc)"]
           REPLICATE --> REL_OLD["pmm::release_frame(old_frame)"]
           REL_OLD --> INC_REP["Increment COW_REPLICATIONS"]
           INC_PROM --> SHOOTDOWN["tlb_shootdown(fault_page)"]
           INC_REP --> SHOOTDOWN
       end

       subgraph Telemetry["Observability Interfaces"]
           SHOOTDOWN --> VMSTAT["/proc/vmstat (pgfault_cow_inplace, nr_cow_active, nr_cow_peak)"]
           SHOOTDOWN --> MEM_CMD["Supervisor 'memory -c' and 'memory -p'"]
       end

2.1 $O(1)$ Hash Table Frame Reference Counting
----------------------------------------------

Physical frame reference counts are tracked using open addressing with linear probing and Knuth backward-shift deletion:

* **Capacity**: 16,384 slots (:math:`2^{14}` entries, 256 KiB in BSS), supporting up to 64 MiB of uniquely shared user frames.
* **Hash Function**:

  .. math::

     \text{slot} = \left(\left(\frac{\text{frame}}{4096}\right) \times \text{0x9E3779B97F4A7C15}\right) \ \& \ 16383

* **Insertion (``retain_frame``)**: If the physical frame is not present in the table, it enters with initial refcount 2 (representing parent and child sharing). If already present, count increments atomically under ``COW_TABLE_LOCK``.
* **Deletion (``release_frame``)**: When refcount drops from 2 to 1 (sole ownership restored), the slot is cleared. Rather than leaving tombstones that degrade subsequent lookups, backward-shift deletion scans contiguous cluster entries and shifts displaced entries closer to their ideal hash index.

2.2 In-Place Promotion vs Frame Replication
-------------------------------------------

When a process attempts to write to a page marked ``PAGE_COW``:

1. If ``frame_refcount(phys_frame) <= 1``:
   All sibling processes have either exited or already duplicated this page. The faulting process is now the sole owner. Keira promotes the page entry in-place by clearing ``PAGE_COW`` and setting ``PAGE_WRITABLE``. No frame allocation and no memory copy occur.
2. If ``frame_refcount(phys_frame) > 1``:
   Multiple processes are actively sharing the frame. Keira allocates a fresh physical frame, copies 4,096 bytes from the existing frame, maps the new frame as writable and calls ``pmm::release_frame(old_frame)``.

3. Telemetry & Userland Observability
=====================================

3.1 Supervisor Shell (memory -c)
--------------------------------

.. code-block:: text

   ring0@keira:/# memory -c
   Copy-On-Write (COW) Memory Telemetry:
     Active Shared Frames : 0 frames (0 KB)
     Peak Shared Frames   : 276 frames (1104 KB)
     Total COW Traps      : 91 faults
     Page Replications    : 46 allocations
     In-Place Promotions  : 45 zero-alloc promotions
     Capacity Limit       : 16384 slots (O(1) Hash Table)

3.2 Dynamic /proc/vmstat Node
-----------------------------

.. code-block:: text

   nr_free_pages 30543
   nr_alloc_pages 318
   nr_page_table_pages 16
   nr_dirty 0
   nr_writeback 0
   nr_slab_reclaimable 0
   nr_slab_unreclaimable 0
   pgpgin 0
   pgpgout 0
   pgfault 4
   pgmajfault 0
   pgfault_cow 0
   pgfault_cow_inplace 0
   pgfault_cow_replicate 0
   nr_cow_active 0
   nr_cow_peak 0
   pgfault_stack 0
   pgfault_demand 0
   tlb_flushes 0

4. Verification & Empirical Results
===================================

The verification battery (``scratch/verify_m49_cow.py``) validated complete end-to-end functionality:

1. **Host Unit Tests**: All 82 unit tests in ``keira-mem`` passed with 0 errors, validating hash collisions, cluster compaction, multi-process refcounts and capacity boundaries.
2. **KCC Compiler**: Native userland C compilation (``/bin/kcc.elf /etc/main.c -o /tmp/hello.elf``) compiled and executed cleanly in Ring 3 userspace.
3. **Network Download**: HTTP fetch (``/bin/fetch.elf``) retrieved synthetic API payload via loopback network stack.
4. **ABI Stress & CoW Churn**: Full execution of ``/bin/test_abi.elf`` (36/36 tests passed) generated 91 CoW faults, 276 peak shared frames (1,104 KiB), 46 replications and 45 zero-allocation in-place promotions, returning to 0 active shared frames with zero memory leaks.
5. **Dual Architecture**: ``make full`` cleanly built kernel and ISO boot artifacts for both ``x86_64`` and ``i686``.
