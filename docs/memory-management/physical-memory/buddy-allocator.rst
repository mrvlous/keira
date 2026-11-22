.. SPDX-License-Identifier: GPL-2.0-only


Binary Buddy Page Allocator & Memory Zones
==========================================

The Physical Memory Management (PMM) subsystem provides a Linux-grade binary buddy page allocator with power-of-two block coalescing and physical memory zone watermarks.

Subsystem Overview
==================

Physical page frames (4 KB base size) are organized into discrete order levels ranging from Order 0 (4 KB) to Order 10 (4 MB):

.. list-table::
   :header-rows: 1
   :widths: 10 15 20 55

   * - Order
     - Pages
     - Chunk Size
     - Typical Purpose
   * - 0
     - 1
     - 4 KB
     - Single virtual memory pages, page tables and userland text
   * - 1
     - 2
     - 8 KB
     - Kernel thread stacks and double-page buffers
   * - 2
     - 4
     - 16 KB
     - Medium slab caches and file descriptors
   * - 3
     - 8
     - 32 KB
     - Network receive queues and pipe buffers
   * - 4
     - 16
     - 64 KB
     - Disk I/O scatter-gather tables
   * - 5-9
     - 32-512
     - 128 KB - 2 MB
     - Huge pages, DMA buffers and disk page cache blocks
   * - 10
     - 1024
     - 4 MB
     - Multi-megabyte contiguous buffers (``MAX_ORDER``)

Buddy Pairing & Coalescing Arithmetic
=====================================

Blocks are paired using bitwise XOR arithmetic on their Page Frame Number (PFN):

.. math::

   \text{buddy\_pfn} = \text{pfn} \oplus 2^{\text{order}}

When a block is deallocated, its buddy block is inspected. If the buddy is free, unallocated and situated at the identical order level, both blocks are combined into an order :math:`k+1` block at base PFN :math:`\text{pfn} \ \& \ \sim(2^{\text{order}})`.

Physical Memory Zones
=====================

Physical memory is partitioned into three functional zones:

* **ZONE_DMA** (0 to 16 MB): Reserved for ISA DMA controllers and legacy peripheral devices limited to 24-bit physical addresses.
* **ZONE_NORMAL** (16 MB to 896 MB): Directly mapped kernel memory for general allocations and process page tables.
* **ZONE_HIGHMEM** (Above 896 MB): Dynamically mapped physical frames on 32-bit platforms where physical address space exceeds direct kernel virtual memory capacity.

Watermark Monitoring & Reclamation
==================================

Each memory zone maintains three watermark thresholds:

* **wmark_min**: 2% of managed pages. When free pages drop below this level, direct reclamation occurs.
* **wmark_low**: 1.25x of ``wmark_min``. Falling below this threshold awakens the background reclamation daemon (``[kswapd0]``).
* **wmark_high**: 1.50x of ``wmark_min``. Once free memory is restored above this threshold, ``[kswapd0]`` returns to idle sleep.

ProcFS Virtual Telemetry
========================

Live buddy allocator fragmentation metrics are exposed via ``/proc/buddyinfo`` in standard Linux tabular format:

.. code-block:: text

   Node 0, zone      DMA      1      0      1      0      2      1      1      0      1      1      3
   Node 0, zone   Normal   3216    245     59     19     12      4      2      1      1      0      0
   Node 0, zone  HighMem      0      0      0      0      0      0      0      0      0      0      0
