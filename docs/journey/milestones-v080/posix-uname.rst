.. SPDX-License-Identifier: GPL-2.0-only


Milestone 55: POSIX System Identity Subsystem, Century-Plus Syscall 108 (SYS_UNAME) & Native uname Utility Refactoring
======================================================================================================================

1. Executive Summary
====================

Milestone 55 introduces the POSIX System Identity Subsystem to the Keira Kernel via Century-Plus Syscall 108 (``SYS_UNAME``). Previously, userland system identification relied on parsing text endpoints in ``/proc/version`` and ``/etc/hostname``. Milestone 55 standardizes kernel identity delivery into a canonical, structured POSIX ``struct utsname`` binary contract, decoupling userspace from virtual filesystem text formatting.

Through Milestone 55, Keira delivers:

1. **Century-Plus Syscall 108 (``SYS_UNAME``)**: Registered across kernel vector routing tables (``crates/syscall/src/table/numbers/system.rs`` and ``crates/syscall/src/table/vectors/names.rs``).
2. **Ring 0 System Identity Engine**: Implemented in ``crates/syscall/src/dispatcher/handlers/system.rs``. The handler dynamically detects target architecture (``x86_64`` vs ``i686``), synchronizes the system node name with ``/etc/hostname`` (falling back to ``"keira"`` if unconfigured) and formats release (``"0.8.0"``) and build version metadata with zero dynamic allocations.
3. **Memory Safety & Pointer Validation**: Strict user pointer validation through ``validate_user_ptr`` ensuring destination memory buffers reside strictly within userland address space, returning ``-EFAULT`` on invalid or null pointers.
4. **C Runtime Header ``<sys/utsname.h>``**: Standard POSIX definitions for ``struct utsname`` (65 bytes per field, matching standard Linux and System V ABI conventions) and ``uname(struct utsname *buf)`` function prototype.
5. **Freestanding C Library ``uname()`` Wrapper**: Implemented in ``userland/libc/src/syscall/sys/utsname.c`` for ``libc.a``, translating POSIX errno returns and standardizing system call vector 108.
6. **Core Utility ``/bin/uname.elf`` Refactoring**: Refactored ``userland/sysutils/uname.c`` to directly invoke ``uname(&u)`` rather than reading procfs and etc files, reducing binary complexity while adhering strictly to POSIX coreutils architecture.
7. **ABI Compliance Test Suite 40**: Expanded ``/bin/test_abi.elf`` with Test 40, validating null pointer rejection (``EFAULT``), memory boundary isolation, sysname (``"Keira"``), release (``"0.8.0"``), version substring (``"SMP"``) and architecture parity across ``x86_64`` and ``i686``.

2. Architecture & Design
========================

.. code-block:: text

   flowchart TD
       subgraph Userspace["Ring 3 Userspace (C Runtime & Utilities)"]
           UNAME_BIN["Core Utility: /bin/uname.elf"]
           TEST_ABI["ABI Suite: /bin/test_abi.elf (Test 40)"]
           LIBC_UNAME["libc.a: uname(struct utsname *buf)"]
           HDR_UNAME["<sys/utsname.h> (struct utsname)"]
           UNAME_BIN --> LIBC_UNAME
           TEST_ABI --> LIBC_UNAME
           LIBC_UNAME --> HDR_UNAME
       end

       subgraph SyscallBoundary["POSIX Syscall ABI Vector 108"]
           INT80["int 0x80 / syscall instruction"]
           ROUTER["Dispatcher Router: Vector 108"]
           VALIDATOR["User Memory Pointer Validation (validate_user_ptr)"]
           LIBC_UNAME --> INT80
           INT80 --> ROUTER
           ROUTER --> VALIDATOR
       end

       subgraph KernelEngine["Ring 0 System Identity Engine"]
           UNAME_HANDLER["handle_uname(buf: usize)"]
           SYSNAME["Kernel Name: 'Keira'"]
           NODENAME["VFS Read: /etc/hostname (fallback: 'keira')"]
           RELEASE["Kernel Release: '0.8.0'"]
           VERSION["Version: '#1 SMP 2026'"]
           MACHINE["Target Arch: 'x86_64' or 'i686'"]
           DOMAIN["Domain Name: ''"]
           COPY_USER["copy_to_user(buf, &uts, sizeof)"]

           VALIDATOR --> UNAME_HANDLER
           UNAME_HANDLER --> SYSNAME
           UNAME_HANDLER --> NODENAME
           UNAME_HANDLER --> RELEASE
           UNAME_HANDLER --> VERSION
           UNAME_HANDLER --> MACHINE
           UNAME_HANDLER --> DOMAIN
           UNAME_HANDLER --> COPY_USER
       end

3. POSIX Binary Contract & Memory Layout
========================================

3.1 Data Structures
-------------------

.. code-block:: c

   #define _UTSNAME_LENGTH 65

   struct utsname {
       char sysname[_UTSNAME_LENGTH];
       char nodename[_UTSNAME_LENGTH];
       char release[_UTSNAME_LENGTH];
       char version[_UTSNAME_LENGTH];
       char machine[_UTSNAME_LENGTH];
       char domainname[_UTSNAME_LENGTH];
   };

In the kernel implementation, ``UtsName`` mirrors this exact memory layout:

.. code-block:: rust

   pub const UTSNAME_FIELD_LEN: usize = 65;

   #[repr(C)]
   #[derive(Copy, Clone)]
   pub struct UtsName {
       pub sysname: [u8; UTSNAME_FIELD_LEN],
       pub nodename: [u8; UTSNAME_FIELD_LEN],
       pub release: [u8; UTSNAME_FIELD_LEN],
       pub version: [u8; UTSNAME_FIELD_LEN],
       pub machine: [u8; UTSNAME_FIELD_LEN],
       pub domainname: [u8; UTSNAME_FIELD_LEN],
   }

3.2 Error Handling & POSIX Invariants
-------------------------------------

- **``EFAULT``**: Returned when ``buf`` is ``NULL`` or points outside valid user virtual address space.
- **Buffer Safety**: All fields are zero-padded to ensure guaranteed null-termination across all architectures.

4. Verification Battery & Test Metrics
======================================

.. list-table::
   :header-rows: 1
   :widths: 20 19 42 17

   * - Suite
     - Component
     - Scope
     - Result
   * - **Kernel Units**
     - ``keira-syscall``
     - Syscall 108 routing and utsname generation
     - **35/35 Passed**
   * - **Userland Utility**
     - ``/bin/uname.elf``
     - Direct invocation (``-a``, ``-s``, ``-r``, ``-m``)
     - **Passed**
   * - **ABI Compliance**
     - ``/bin/test_abi.elf``
     - Test 40 (sys_uname identity verification)
     - **40/40 Passed**
   * - **Compiler Test**
     - ``/bin/kcc.elf``
     - Native C compilation and execution
     - **Passed**
   * - **Network Engine**
     - ``/bin/fetch.elf``
     - Live HTTP fetch over pure BSD sockets
     - **Passed**
   * - **Multi-Arch Build**
     - ``make full``
     - Both ``x86_64`` and ``i686`` targets
     - **0 err, 0 warn**
