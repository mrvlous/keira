.. SPDX-License-Identifier: GPL-2.0-only


Ring 0 Supervisor & Optional Ring 3 Userland Demarcation
========================================================

Keira enforces an austere architectural demarcation between the Ring 0 supervisor kernel and the optional Ring 3 userspace environment. Following the classical monolithic design of Linux and Unix kernels, Ring 0 operates autonomously from first principles, managing physical hardware, memory paging, interrupt vectors and preemptive task scheduling. Ring 3 userspace is strictly an optional reference runtime and conformance test suite.

1. Architectural Philosophy & Pure Kernel Autonomy
==================================================

Keira is fundamentally a freestanding monolithic kernel in Ring 0, not an operating system distribution. The kernel maintains absolute independence from any userland package, daemon or binary disk:

- **Autonomous Supervisor Operation (USERLAND=0)**: When built with ``USERLAND=0`` (or booted via ``make run-bare`` / ``make pure_defconfig``), Keira compiles into a standalone pure kernel ISO (``keira-bare-<arch>-*.iso``) without any userland disk, ``initrd.tar`` or ``/etc/*`` files. The kernel initializes physical memory, devices, SMP cores and lands directly into the Ring 0 supervisor console (``ring0@keira:/#``).
- **Optional Reference Userspace (USERLAND=1)**: When enabled with ``USERLAND=1``, the build system compiles the freestanding C runtime (``libc.a``), the native compiler (``kcc.elf``) and core diagnostic utilities into a root filesystem disk image. This layer serves as an official ABI conformance test suite to stress-test system call vectors, memory protections, BSD sockets and asynchronous I/O.
- **Zero Distro Bloat Invariant**: Keira strictly omits traditional distribution metadata, package databases, issue banners, fake user tables (``/etc/passwd``) and PAM modules. Only baseline POSIX node resolution files (``/etc/hostname`` and ``/etc/hosts``) are staged when userland is enabled.

.. code-block:: text

   +-----------------------------------------------------------------------+
   |                       RING 0 SUPERVISOR KERNEL                        |
   |   - Memory Management (PMM / VMM / CoW / Slab / Page Cache)           |
   |   - Preemptive Multi-Core Scheduler & SMP AP Trampoline               |
   |   - Virtual Filesystem (VFS) Router, DevFS, ProcFS, FAT & Ext4        |
   |   - Network Protocol Stack (L2-L4), BSD Sockets & Packet Filtering    |
   |   - In-Kernel Diagnostic Console (ring0@keira:/#)                     |
   +-----------------------------------------------------------------------+
                                      |
                     Hardware Privilege Barrier (GDT / MSRs)
                                      |
   +-----------------------------------------------------------------------+
   |             OPTIONAL RING 3 REFERENCE USERSPACE (userland/)           |
   |   - Freestanding C Runtime Library (libc.a & crt0.s)                  |
   |   - Interactive Command Interpreter (sh.elf) with Built-in Help       |
   |   - 53 Native Utility Binaries, Compiler (kcc.elf) & Test Suites      |
   +-----------------------------------------------------------------------+

2. Hardware Privilege Boundary & Processor Control
==================================================

The processor hardware enforces privilege separation using x86 Protection Rings. Ring 0 operates with Current Privilege Level 0 (CPL 0), granting unrestricted access to control registers (CR0, CR3, CR4), Model Specific Registers (MSRs) and I/O ports. Ring 3 operates at CPL 3, restricting code execution to userland instructions:

.. list-table::
   :header-rows: 1
   :widths: 22 20 58

   * - Descriptor / Register
     - Privilege Ring
     - Hardware Configuration and Enforcement
   * - Kernel Code Segment
     - Ring 0 (CPL 0)
     - Selector ``0x08`` (GDT index 1), base 0, limit 4GB, DPL 0.
   * - Kernel Data Segment
     - Ring 0 (CPL 0)
     - Selector ``0x10`` (GDT index 2), base 0, limit 4GB, DPL 0.
   * - User Code Segment
     - Ring 3 (CPL 3)
     - Selector ``0x1B`` (GDT index 3, RPL 3), base 0, limit 4GB, DPL 3.
   * - User Data Segment
     - Ring 3 (CPL 3)
     - Selector ``0x23`` (GDT index 4, RPL 3), base 0, limit 4GB, DPL 3.
   * - Task State Segment (TSS)
     - Ring 0 Hook
     - Stores ``rsp0`` (x86_64) or ``esp0`` (i686) used by CPU hardware on privilege transitions.

System call transitions cross the privilege boundary via hardware-accelerated vectors:

- **64-bit Long Mode (x86_64)**: Fast system call entry via the ``syscall`` instruction. The kernel configures ``IA32_STAR`` with user and kernel segment selectors, ``IA32_LSTAR`` with the entry point address and ``IA32_FMASK`` to mask interrupt flags (IF) upon entry. The return to Ring 3 is executed via ``sysretq``.
- **32-bit Protected Mode (i686)**: Software interrupt dispatch via ``int 0x80`` through the Interrupt Descriptor Table (IDT). The hardware switches stack pointers using ``TSS.esp0`` and saves user CS, EIP, EFLAGS, SS and ESP on the kernel stack before dispatching to the system call router.

3. Virtual Address Space Partitioning & Memory Protection
=========================================================

Keira strictly isolates virtual memory spaces between supervisor and userspace:

.. code-block:: text

   x86_64 Virtual Address Space Layout:
   +--------------------------------------+ 0xFFFF_FFFF_FFFF_FFFF
   | Higher-Half Kernel Space (Ring 0)    |
   | - Kernel Image (.text, .rodata, .bss)|
   | - Direct Physical Map & Page Tables  |
   | - Kernel Stacks & Heap Allocators    |
   +--------------------------------------+ 0xFFFF_8000_0000_0000
   | Non-Canonical Canonical Address Hole |
   +--------------------------------------+ 0x0000_7FFF_FFFF_FFFF
   | User Space Address Range (Ring 3)    |
   | - User Stack (grows downwards)       |
   | - Heap Space (brk allocator)         |
   | - User Code (.text) & Data (.data)   |
   +--------------------------------------+ 0x0000_0000_0040_0000

Memory protection is enforced at the page table level:

- **User/Supervisor Bit (U/S)**: Kernel pages have the U/S bit cleared (0), forbidding any Ring 3 read or write access. An attempt by user code to read higher-half kernel memory triggers a Page Fault exception (#PF) with error code protection violation.
- **Write Protect & Execute Disable (W^X)**: Executable code pages are marked read-only. Data, heap and stack pages are marked non-executable using the No-Execute (NX) bit on modern processors.
- **Address Space Layout Randomization (ASLR)**: Base addresses for user code, heap and stack are randomized using entropy gathered from hardware silicon sources (RDRAND/RDSEED) and CMOS RTC timing jitter.

4. Safe User Memory Validation Primitives
=========================================

The kernel never dereferences userland pointers directly within Ring 0. All user-supplied buffers, strings and structs passed via system call registers undergo rigorous boundary verification:

1. **Pointer Validation (``validate_user_ptr``)**: Verifies that the pointer falls strictly within the lower-half user virtual address space range and does not cross into supervisor memory.
2. **Safe Copy In (``copy_from_user``)**: Bounded memory copy that verifies each virtual page is mapped and readable by userland before copying bytes into kernel memory.
3. **Safe Copy Out (``copy_to_user``)**: Bounded memory copy that verifies target pages are writable and user-accessible before delivering kernel data to userland buffers.

5. Exception Containment & Fault Containment
============================================

Ring 3 faults are strictly isolated from Ring 0:

- **General Protection Faults (#GP)**: Triggered if userland attempts to execute privileged instructions (such as ``cli``, ``sti``, ``hlt`` or ``in``/``out`` without IOPL permissions). The kernel captures the fault, generates a ``SIGSEGV`` or ``SIGILL`` signal and terminates the offending process.
- **Page Faults (#PF)**: If userland accesses unmapped memory, the kernel page fault handler checks for Copy-on-Write (CoW) page duplication or demand paging. If the access is invalid, the kernel raises ``SIGSEGV`` and produces a crash telemetry report in ``/var/log/``.
- **Kernel Survivability**: A crashed, aborted or malicious userland process cannot compromise kernel stability. Kernel data structures, scheduler queues and hardware devices remain fully operational.

6. Clean Transition Handover to Supervisor Console
==================================================

When the userspace shell (``/bin/sh.elf``) completes execution (for example, when the user inputs ``exit``), the kernel cleanly reclaims the process address space and yields control back to the Ring 0 supervisor console:

.. code-block:: text

   sequenceDiagram
       participant Ring3 as Ring 3 (/bin/sh.elf)
       participant Syscall as Syscall Vector (exit)
       participant Sched as Task Scheduler
       participant Supervisor as Ring 0 Supervisor Console

       Ring3->>Syscall: invoke sys_exit(status)
       Syscall->>Sched: reap task resources & destroy user VMA
       Sched->>Supervisor: transition control to enter_main_loop()
       Supervisor->>Supervisor: print supervisor prompt (ring0@keira:/#)

The supervisor console provides immediate, low-level hardware diagnostics, memory inspection commands and manual binary execution without requiring reboot or system restart.
