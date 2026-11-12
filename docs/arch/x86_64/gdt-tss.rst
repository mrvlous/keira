.. SPDX-License-Identifier: GPL-2.0-only


Global Descriptor Table (GDT) & Task State Segment (TSS)
========================================================

Privilege domains and segment descriptors in IA-32e Long Mode are handled in ``arch/x86/x86_64/kernel/gdt.asm`` and ``crates/kernel/src/arch/x86_64/gdt.rs``.

In 64-bit mode, hardware segmentation base and limit enforcement is disabled for code and data segments (treating bases as ``0``), but descriptors remain mandatory for Ring privilege transitions (Ring 0 vs Ring 3) and ``sysret`` alignment.

1. 64-bit GDT Selector Topology
===============================

.. list-table:: 64-bit GDT Selectors
   :widths: 15 15 20 50
   :header-rows: 1

   * - Selector
     - RPL / DPL
     - Type
     - Architectural Purpose
   * - ``0x00``
     - Ring 0
     - Null
     - Mandatory architecture null descriptor.
   * - ``0x08``
     - Ring 0
     - 64-bit Code
     - Kernel code execution (``CS.L = 1``).
   * - ``0x10``
     - Ring 0
     - 64-bit Data
     - Kernel data segment (``DS``, ``SS``, ``ES``).
   * - ``0x18``
     - Ring 3
     - 64-bit Data
     - User data segment.
   * - ``0x20``
     - Ring 3
     - 64-bit Data
     - User data target for ``sysret``.
   * - ``0x28``
     - Ring 3
     - 64-bit Code
     - User code target for ``sysret``.
   * - ``0x30``
     - Ring 0
     - 16-byte TSS
     - 64-bit Task State Segment descriptor holding ``RSP0`` and IST pointers.

2. Task State Segment (TSS) & IST
=================================

The 64-bit TSS does not perform hardware task switching. Instead, it provides:

* **``RSP0``**: The kernel stack pointer loaded upon privilege transition from Ring 3 to Ring 0.
* **Interrupt Stack Table (IST)**: Up to 7 dedicated stack pointers for critical exceptions (e.g. Double Fault ``#DF`` and NMI) to prevent stack overflow cascading.
