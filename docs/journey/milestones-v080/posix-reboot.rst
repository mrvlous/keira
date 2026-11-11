.. SPDX-License-Identifier: GPL-2.0-only

=======================================================================================================================
Milestone 56: POSIX System Power Management Subsystem, Century-Plus Syscall 109 (SYS_REBOOT) & Core Utilities Expansion
=======================================================================================================================

1. Executive Summary
====================

Milestone 56 introduces the POSIX System Power Management and Control Subsystem to Keira Kernel via Century-Plus Syscall 109 (``SYS_REBOOT``). Previously, system shutdown and reboot operations required dropping out of user space into the Ring 0 supervisor console. Milestone 56 bridges the kernel's low-level ACPI power registers and keyboard controller reset logic directly to the POSIX C userland environment, providing standard machine control utilities while enforcing strict Linux-compatible security boundaries.

Through Milestone 56, Keira delivers:

1. **Century-Plus Syscall 109 (``SYS_REBOOT``)**: System call number ``SYS_REBOOT = 109`` registered across kernel vector routing tables (``crates/syscall/src/table/numbers/system.rs`` and ``crates/syscall/src/table/vectors/names.rs``).
2. **Linux-Compatible Security & Magic Number Protection**: Enforces cryptographic magic parameters (``LINUX_REBOOT_MAGIC1 = 0xfee1dead`` and ``LINUX_REBOOT_MAGIC2 = 672274793``) to prevent accidental or malicious triggering by corrupted memory addresses, returning ``-EINVAL`` on parameter mismatch.
3. **Privilege Boundary Enforcement**: Syscall access is restricted strictly to root processes (``uid == 0``). Invocations from unprivileged accounts return ``-EPERM``.
4. **Ring 0 Power Dispatch Engine**: Seamlessly transitions execution into ``keira_arch::power::acpi::reboot()`` or ``keira_arch::power::acpi::poweroff()`` based on the command argument (``RB_AUTOBOOT`` vs ``RB_POWER_OFF``), performing platform register sequencing across ACPI sleep state S5 and 8042/PCI reset ports.
5. **C Runtime Header ``<sys/reboot.h>``**: Provides standard POSIX definitions for reboot magic numbers, command flags (``RB_AUTOBOOT``, ``RB_HALT_SYSTEM``, ``RB_POWER_OFF``) and function prototypes (``reboot()`` and ``sys_reboot()``).
6. **Freestanding C Library ``reboot()`` Wrapper**: Implemented in ``userland/lib/syscall/sys/reboot.c`` for ``libc.a``, translating errno returns and standardizing userland system calls.
7. **Standalone Core Utilities (``/bin/reboot.elf`` & ``/bin/poweroff.elf``)**: Implementation of ``/bin/reboot.elf`` and ``/bin/poweroff.elf``, expanding Keira's core standalone utility inventory to 46 native binaries. Both utilities support clean disk cache flushing (``sync()``) and forceful immediate termination (``-f``/``--force``).
8. **ABI Compliance Test Suite 41**: Expanded ``/bin/test_abi.elf`` with Test 41, verifying magic number containment, invalid command rejection and privilege boundary isolation.

2. Architecture & Design
========================

.. code-block:: text

   flowchart TD
       subgraph Userspace["Ring 3 Userspace (C Runtime & Utilities)"]
           BIN_REBOOT["Core Utility: /bin/reboot.elf (45th)"]
           BIN_POWEROFF["Core Utility: /bin/poweroff.elf (46th)"]
           TEST_ABI["ABI Suite: /bin/test_abi.elf (Test 41)"]
           LIBC_REBOOT["libc.a: reboot(int cmd)"]
           HDR_REBOOT["<sys/reboot.h> (Magic Constants & Commands)"]

           BIN_REBOOT --> LIBC_REBOOT
           BIN_POWEROFF --> LIBC_REBOOT
           TEST_ABI --> LIBC_REBOOT
           LIBC_REBOOT --> HDR_REBOOT
       end

       subgraph SyscallBoundary["POSIX Syscall ABI Vector 109"]
           INT80["int 0x80 / syscall instruction"]
           ROUTER["Dispatcher Router: Vector 109"]
           VALIDATOR["Parameter & Privilege Verification"]

           LIBC_REBOOT --> INT80
           INT80 --> ROUTER
           ROUTER --> VALIDATOR
       end

       subgraph KernelEngine["Ring 0 Power Control Subsystem"]
           HANDLER["handle_reboot(magic1, magic2, cmd, arg)"]
           UID_CHECK["Root Privilege Check (get_current_uid == 0)"]
           MAGIC_CHECK["Magic Numbers: 0xfee1dead & 672274793"]
           ACPI_REBOOT["keira_arch::power::acpi::reboot()"]
           ACPI_POWEROFF["keira_arch::power::acpi::poweroff()"]

           VALIDATOR --> HANDLER
           HANDLER --> UID_CHECK
           UID_CHECK --> MAGIC_CHECK
           MAGIC_CHECK --> ACPI_REBOOT
           MAGIC_CHECK --> ACPI_POWEROFF
       end

3. POSIX Binary Contract & Magic Constants
==========================================

3.1 Magic Constants
-------------------

.. code-block:: c

   #define LINUX_REBOOT_MAGIC1     0xfee1dead
   #define LINUX_REBOOT_MAGIC2     672274793
   #define LINUX_REBOOT_MAGIC2A    85072278
   #define LINUX_REBOOT_MAGIC2B    369367448
   #define LINUX_REBOOT_MAGIC2C    537993216

   #define RB_AUTOBOOT             0x01234567
   #define RB_HALT_SYSTEM          0xcdef0123
   #define RB_POWER_OFF            0x4321fedc

3.2 Error Handling & POSIX Invariants
-------------------------------------

- **``EPERM``**: Returned when an unprivileged process (``uid != 0``) invokes ``sys_reboot``.
- **``EINVAL``**: Returned when ``magic1`` or ``magic2`` do not match canonical values or when ``cmd`` is unrecognized.

4. Verification Battery & Test Metrics
======================================

.. list-table::
   :header-rows: 1
   :widths: 22 38 50 17

   * - Suite
     - Component
     - Scope
     - Result
   * - **Kernel Units**
     - ``keira-syscall``
     - Syscall 109 routing, magic validation and commands
     - **36/36 Passed**
   * - **Userland Utilities**
     - ``/bin/reboot.elf``, ``/bin/poweroff.elf``
     - Direct invocation (``-h``, ``--help``)
     - **Passed**
   * - **ABI Compliance**
     - ``/bin/test_abi.elf``
     - Test 41 (sys_reboot boundary containment)
     - **41/41 Passed**
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
