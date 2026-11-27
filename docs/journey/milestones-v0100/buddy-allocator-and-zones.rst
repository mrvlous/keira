.. SPDX-License-Identifier: GPL-2.0-only


Milestone 66: Linux-Grade Binary Buddy Allocator, Memory Zone Architecture & Dynamic /proc/buddyinfo Telemetry
==============================================================================================================

Milestone 66 introduces a Linux-grade binary buddy page allocator engine, physical memory zone architecture (``ZONE_DMA``, ``ZONE_NORMAL`` and ``ZONE_HIGHMEM``) and dynamic ``/proc/buddyinfo`` virtual telemetry to Keira's Ring 0 memory subsystem. Building upon the autonomous background worker daemons of Milestone 64 and developer introspection tooling of Milestone 65, this milestone replaces flat contiguous bitmap searches with power-of-two chunk splitting, bitwise XOR buddy pairing and recursive coalescing.

1. Architectural Motivation & Fragmentation Invariants
======================================================

As a freestanding monolithic kernel in Ring 0, Keira manages hardware physical memory from first principles across both modern 64-bit (``x86_64``) and pure 32-bit (``i686``) bare-metal targets:

1. **External Memory Fragmentation**: Repeated allocations and deallocations of disparate page frame lengths create non-contiguous free memory holes. Without structured order bucketing, allocating multi-page contiguous buffers (e.g. for network rings, DMA or disk pipelines) requires expensive linear bitmap traversals and can fail despite abundant free memory.
2. **Deterministic Power-of-Two Allocation**: Categorizing free physical frames into discrete order levels (Order 0 for 4 KB up to Order 10 for 4 MB) provides :math:`O(1)` allocation of power-of-two contiguous chunks.
3. **Hardware DMA and Direct Map Demarcation**: Hardware devices, legacy ISA controllers and 32-bit CPU architectures impose rigid physical addressing boundaries. Physical memory must be partitioned into hardware-aware zones with dedicated watermark monitoring.
4. **Pure Kernel Primacy**: The buddy engine, zone descriptors and watermark calculators execute strictly in Ring 0 and require zero userland binaries or storage devices to operate during ``make run-bare``.

2. Binary Buddy Allocator Engine (``crates/mem/src/pmm/buddy.rs``)
==================================================================

The binary buddy allocator organizes physical memory into 11 discrete order levels (0 through 10, representing :math:`2^0` to :math:`2^{10}` pages or 4 KB to 4 MB chunks):

* **Bitwise Buddy Address Pairing**: For any given block at Page Frame Number (PFN) :math:`P` of order :math:`k`, its unique buddy block resides at :math:`P \oplus 2^k`.
* **Power-of-Two Block Splitting**: When an allocation request arrives for order :math:`k` and the free list at order :math:`k` is empty, the allocator scans higher orders :math:`k+1 \le j \le 10`. The lowest available block at order :math:`j` is recursively split in halves, with buddy blocks enqueued into intermediate order lists until the requested block is produced.
* **Recursive Coalescing**: When a block of order :math:`k` is freed, the engine checks whether its buddy at :math:`P \oplus 2^k` is currently free and at the identical order level. If so, both blocks are merged into an order :math:`k+1` block at base PFN :math:`P \ \& \ \sim(2^k)`. This coalescing cascades recursively up to ``MAX_ORDER`` (Order 10).
* **Order Nibble Map**: A compact tracking table of 4-bit nibbles records the active order tag of every allocated or free block header across up to 4 GB of addressable physical RAM (1,048,576 frames).

3. Physical Memory Zone Architecture & Watermarks (``crates/mem/src/pmm/zone.rs``)
==================================================================================

Physical address space is categorized into three standard monolithic zones:

.. list-table::
   :header-rows: 1
   :widths: 15 25 60

   * - Zone
     - Physical Range
     - Functional Role
   * - ``ZONE_DMA``
     - 0 to 16 MB (PFN 0 to 4096)
     - Reserved for ISA DMA controllers and legacy peripheral devices limited to 24-bit addressing.
   * - ``ZONE_NORMAL``
     - 16 MB to 896 MB (PFN 4096 to 229376)
     - Direct kernel mapped memory for general kernel allocations, page tables and userland address spaces.
   * - ``ZONE_HIGHMEM``
     - Above 896 MB (PFN 229376+)
     - Dynamically mapped physical memory on 32-bit platforms exceeding direct kernel virtual address limits.

Each zone maintains dedicated watermark thresholds computed dynamically during boot:

* **Watermark Min (``wmark_min``)**: 2% of managed pages (minimum 16 frames). When free pages drop below this threshold, direct synchronous memory reclamation is triggered.
* **Watermark Low (``wmark_low``)**: ``wmark_min * 1.25``. When free pages fall below this watermark, the background page reclamation daemon (``[kswapd0]``) is activated.
* **Watermark High (``wmark_high``)**: ``wmark_min * 1.50``. When background reclamation restores free pages above this threshold, ``[kswapd0]`` returns to sleep.

4. Dynamic ProcFS Telemetry (``/proc/buddyinfo``)
=================================================

The ``/proc/buddyinfo`` virtual file exposes live buddy allocator chunk fragmentation matching the Linux kernel format:

.. code-block:: text

   Node 0, zone      DMA      1      0      1      0      2      1      1      0      1      1      3
   Node 0, zone   Normal   3216    245     59     19     12      4      2      1      1      0      0
   Node 0, zone  HighMem      0      0      0      0      0      0      0      0      0      0      0

Each numerical column indicates the exact count of free blocks available at order 0 (4 KB), order 1 (8 KB), order 2 (16 KB) through order 10 (4 MB).

5. Supervisor Console Integration (``buddyinfo`` & ``memory -z``)
=================================================================

The Ring 0 supervisor console integrates dedicated commands for live introspection:

* ``buddyinfo``: Displays tabular free block counts per order across all active zones.
* ``buddyinfo -v``: Displays chunk byte sizes, page counts and capacities for each order hierarchy.
* ``buddyinfo -z <zone>``: Filters output specifically to DMA, Normal or HighMem.
* ``memory -z``: Displays physical memory zone descriptors, managed page totals and watermark thresholds.

6. Invariants, Dual-Arch Parity & Verification
==============================================

The binary buddy allocator adheres to strict kernel invariants:

* **Dual-Architecture Parity**: Zero compilation warnings, clippy errors or test failures on both ``x86_64`` and ``i686``.
* **Pure Kernel Primacy**: The system boots autonomously and executes all memory diagnostics with ``make run-bare``.
* **Zero Host Test Memory Corruption**: Intrusive linked list operations utilize static mock node tables during host unit testing (``cargo test``), ensuring leak-free verification.
