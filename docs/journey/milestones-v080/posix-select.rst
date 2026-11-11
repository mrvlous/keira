.. SPDX-License-Identifier: GPL-2.0-only

======================================================================================================================
Milestone 53: POSIX Synchronous I/O Multiplexing Triad Completion, SYS_SELECT (Vector 107) & Descriptor Bitmask Engine
======================================================================================================================

1. Executive Summary
====================

Milestone 53 completes the POSIX synchronous I/O multiplexing triad in Keira Kernel (``epoll`` -> ``poll`` -> ``select``) via Century-Plus Syscall 107 (``SYS_SELECT``). While ``epoll`` provides high-performance asynchronous event notifications for scalable servers and ``poll`` enables structured array descriptor polling, the classic ``select()`` interface remains a fundamental building block for standard C runtime libraries, event loops, curses interfaces and portable UNIX system software.

Through Milestone 53, Keira introduces:

1. **Century-Plus Syscall 107 (``SYS_SELECT``)**: System call number ``SYS_SELECT = 107`` defined across kernel vector tables, vector resolution routines and architecture headers.
2. **Descriptor Bitmask Engine (``FdSet``)**: A high-efficiency readiness evaluator in ``crates/syscall/src/dispatcher/handlers/select.rs`` supporting standard POSIX bitmasks (``readfds``, ``writefds`` and ``exceptfds``). The engine evaluates readiness across anonymous pipes, UNIX domain sockets (``AF_UNIX``), network sockets (TCP/UDP), pseudo-terminals and virtual filesystem inodes.
3. **Timed Multiplexing & Microsecond Delays**: Support for immediate non-blocking status evaluation (``timeout->tv_sec == 0 && timeout->tv_usec == 0``), indefinite blocking (``timeout == NULL``), high-resolution microsecond timeouts and pure sleep optimization (``select(0, NULL, NULL, NULL, &tv)``) calibrated with HPET timers.
4. **Supervisor ``selectinfo`` Diagnostic Command**: Diagnostic inspection command in ``crates/shell`` displaying real-time multiplexing invocations, ready signals, bad descriptor rejections and timeout telemetry.
5. **Freestanding C Library ``<sys/select.h>`` & ``select()``**: Integration of ``<sys/select.h>`` and ``<select.h>`` into Keira's freestanding C standard library, implementing standard ``fd_set``, bit-manipulation macros (``FD_ZERO``, ``FD_SET``, ``FD_CLR`` and ``FD_ISSET``) and ``select()``.
6. **Standalone ``/bin/test_select`` Utility**: Implementation of ``/bin/test_select.elf``, expanding Keira's standalone core utility suite to 44 binaries.
7. **ABI Compliance Test Suite Expansion (Test 39)**: Addition of Test 39 to ``/bin/test_abi.elf``, verifying pipe readiness transitions, unopened descriptor ``EBADF`` rejection, microsecond timeout expiration, high-resolution null sleep and peer disconnect EOF reporting.

2. Architecture & Design
========================

.. code-block:: text

   flowchart TD
       subgraph Userspace["Ring 3 Userspace (C Runtime & Utilities)"]
           TEST_ABI["ABI Suite: /bin/test_abi.elf (Test 39)"]
           TEST_SELECT["Core Utility: /bin/test_select.elf (44th Binary)"]
           LIBC_SELECT["libc.a: select(nfds, readfds, writefds, exceptfds, timeout)"]
           HDR_SELECT["<sys/select.h> & <select.h>"]
           TEST_ABI --> LIBC_SELECT
           TEST_SELECT --> LIBC_SELECT
           LIBC_SELECT --> HDR_SELECT
       end

       subgraph SyscallBoundary["POSIX Syscall ABI Vector 107"]
           INT80["int 0x80 / syscall instruction"]
           ROUTER["Dispatcher Router: Vector 107"]
           VALIDATOR["User Memory Pointer Validation (validate_user_ptr)"]
           LIBC_SELECT --> INT80
           INT80 --> ROUTER
           ROUTER --> VALIDATOR
       end

       subgraph KernelEngine["Ring 0 Bitmask Multiplexing Engine"]
           SELECT_HANDLER["handle_select(nfds, rfds, wfds, efds, timeout)"]
           READINESS["Descriptor Readiness Inspection (Pipes, Sockets, TTY, VFS)"]
           TELEMETRY["SELECT_TELEMETRY (Calls, Signals, Timeouts, EBADF)"]
           HPET_TIMER["HPET Hardware Timer Calibration"]
           VALIDATOR --> SELECT_HANDLER
           SELECT_HANDLER --> READINESS
           SELECT_HANDLER --> TELEMETRY
           SELECT_HANDLER --> HPET_TIMER
       end

       subgraph Supervisor["Ring 0 Supervisor Shell"]
           CMD_SELECT["selectinfo / select command"]
           CMD_SELECT --> TELEMETRY
       end

3. Kernel Bitmask Engine Specification
======================================

3.1 Data Structures
-------------------

.. code-block:: rust

   pub const FD_SETSIZE: usize = 1024;
   pub const FD_SET_WORDS: usize = 16;

   #[repr(C)]
   #[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
   pub struct FdSet {
       pub bits: [u64; FD_SET_WORDS],
   }

   #[repr(C)]
   #[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
   pub struct TimeVal {
       pub tv_sec: i64,
       pub tv_usec: i64,
   }

3.2 Error Handling & POSIX Invariants
-------------------------------------

- **``EINVAL``**: Returned when ``nfds > 1024`` or when ``tv_sec < 0``, ``tv_usec < 0`` or ``tv_usec >= 1_000_000``.
- **``EFAULT``**: Returned when a non-null bitmask pointer references inaccessible or unmapped user virtual memory.
- **``EBADF``**: Returned when an unopened or invalid file descriptor (``fd >= MAX_FDS`` or ``!desc.is_open``) is specified in any set.

4. Verification Battery & Test Metrics
======================================

.. list-table::
   :header-rows: 1
   :widths: 20 22 44 17

   * - Suite
     - Component
     - Scope
     - Result
   * - **Kernel Units**
     - ``keira-syscall``
     - Syscall 107 routing and bitmask operations
     - **34/34 Passed**
   * - **Shell Units**
     - ``keira-shell``
     - ``selectinfo`` command, help and autocomplete
     - **54/54 Passed**
   * - **Userland Test**
     - ``/bin/test_select.elf``
     - Dedicated 5-scenario functional verification
     - **Passed**
   * - **ABI Compliance**
     - ``/bin/test_abi.elf``
     - Test 39 (sys_select multiplexing)
     - **39/39 Passed**
   * - **Multi-Arch Build**
     - ``make full``
     - Both ``x86_64`` and ``i686`` targets
     - **0 err, 0 warn**
