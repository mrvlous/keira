.. SPDX-License-Identifier: GPL-2.0-only

============================================
Fast System Call Dispatch (SYSCALL / SYSRET)
============================================

Low-latency Ring 3 to Ring 0 transitions in 64-bit mode bypass legacy interrupt gates (``int 0x80``) and utilize AMD64 fast system call instructions implemented in ``arch/x86/x86_64/kernel/syscall.asm``.

1. MSR Initialization
=====================

During early architecture initialization, the following Model-Specific Registers (MSRs) are configured:

* **``IA32_EFER`` (``0xC0000080``)**: Sets bit 0 (``SCE`` - System Call Enable).
* **``IA32_STAR`` (``0xC0000081``)**:
  * Bits ``47:32``: Kernel Code/Data Selectors (``0x08`` / ``0x10``).
  * Bits ``63:48``: User Code/Data Selectors (``0x18`` / ``0x20``).
* **``IA32_LSTAR`` (``0xC0000082``)**: Target 64-bit linear RIP for the kernel entry routine ``syscall_entry``.
* **``IA32_FMASK`` (``0xC0000084``)**: Mask applied to ``RFLAGS`` upon entry (clears ``IF``, ``TF`` and ``DF``).

2. Syscall Register Conventions
===============================

Conforming to the AMD64 ABI:

.. list-table:: Register Allocation on Syscall Entry
   :widths: 20 80
   :header-rows: 1

   * - Register
     - Function
   * - ``RAX``
     - System call vector number (0 - 88)
   * - ``RDI``
     - First parameter
   * - ``RSI``
     - Second parameter
   * - ``RDX``
     - Third parameter
   * - ``R10``
     - Fourth parameter (replaces ``RCX`` which is overwritten by hardware)
   * - ``R8``
     - Fifth parameter
   * - ``R9``
     - Sixth parameter
   * - ``RCX``
     - Preserves userland return ``RIP`` (hardware saved)
   * - ``R11``
     - Preserves userland ``RFLAGS`` (hardware saved)

3. Subsystem Handler Dispatch
=============================

Upon entering Ring 0, ``syscall_entry`` validates user pointers, saves user context, switches to the kernel stack and branches into ``sys_call_table`` in ``crates/kernel/src/syscall/mod.rs``.
