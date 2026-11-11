.. SPDX-License-Identifier: GPL-2.0-only

==============================
Syscall Router & Entry Routing
==============================

System call invocations on 32-bit Keira use software interrupt vector ``0x80`` implemented in ``arch/x86/i686/kernel/syscall.asm``.

1. Register Calling Conventions
===============================

Conforming to traditional 32-bit Linux syscall ABI:

.. list-table:: 32-bit Syscall Register Mapping
   :widths: 25 75
   :header-rows: 1

   * - Register
     - Content / Meaning
   * - ``EAX``
     - Syscall Vector Number (0 - 88)
   * - ``EBX``
     - First argument
   * - ``ECX``
     - Second argument
   * - ``EDX``
     - Third argument
   * - ``ESI``
     - Fourth argument
   * - ``EDI``
     - Fifth argument
   * - ``EBP``
     - Sixth argument

Return codes are placed back into ``EAX`` before returning via ``iret``.

2. Subsystem Handler Routing
============================

The master router arm dispatches the syscall number to specialized subsystem handlers:

* Filesystem handlers (``fs.rs``)
* Process & Task handlers (``proc.rs``)
* Network handlers (``net.rs``)
* IPC handlers (``ipc.rs``)
* System info handlers (``sys.rs``)
