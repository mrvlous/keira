.. SPDX-License-Identifier: GPL-2.0-only

========================================
Architecture Constants and Hardware MMIO
========================================

Hardware definitions, physical memory offsets and CPU control masks are maintained
centrally in ``arch/x86/common/include/constants.inc``.

Memory and Hardware Layout
==========================

.. list-table:: Architectural Memory Constants
   :widths: 30 25 45
   :header-rows: 1

   * - Identifier
     - Value
     - Architectural Purpose
   * - ``KERNEL_PHYS_BASE``
     - ``0x00100000`` (1 MiB)
     - Kernel physical load base address, bypassing legacy BIOS memory holes.
   * - ``KERNEL_STACK_SIZE``
     - ``0x8000`` (32 KiB)
     - Early bootstrap execution stack reserved for CPU bringup.
   * - ``VGA_BUFFER_ADDR``
     - ``0x000B8000``
     - Physical MMIO text console buffer (80 columns x 25 rows).
   * - ``COM1_PORT``
     - ``0x03F8``
     - Base I/O port for standard 16550 UART serial diagnostics.

CPU Control Bitmasks and Registers
==================================

Control Registers
-----------------

* **``CR0_PROTECTED_BIT`` (Bit 0)**: Protected mode enable (``PE``).
* **``CR0_PAGING_BIT`` (Bit 31)**: Hardware paging activation (``PG``).
* **``CR4_PAE_BIT`` (Bit 5)**: Physical Address Extension; mandatory prerequisite for Long Mode.

Model-Specific Registers (MSR)
------------------------------

* **``EFER_MSR`` (``0xC0000080``)**: Extended Feature Enable Register.
  * ``EFER_LONG_MODE_BIT`` (Bit 8): Long Mode Enable (``LME``).
  * ``EFER_NXE_BIT`` (Bit 11): No-Execute memory protection bit (``NXE``).

Page Table Attributes
---------------------

* ``PAGE_PRESENT`` (Bit 0): Entry is mapped to valid physical memory.
* ``PAGE_WRITABLE`` (Bit 1): Read/Write permission bit.
* ``PAGE_HUGE`` (Bit 7): Designates 2 MiB large page in Page Directory entries.
