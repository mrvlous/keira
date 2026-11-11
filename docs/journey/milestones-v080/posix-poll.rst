.. SPDX-License-Identifier: GPL-2.0-only

========================================================================================================
Milestone 52: POSIX Synchronous I/O Multiplexing, SYS_POLL (Vector 106) & File Descriptor Polling Engine
========================================================================================================

1. Executive Summary
====================

Milestone 52 implements POSIX synchronous I/O multiplexing in Keira Kernel via Century-Plus Syscall 106 (``SYS_POLL``). Prior to this milestone, userland processes requiring multi-descriptor event monitoring relied exclusively on the event notification mechanism (``epoll``). While ``epoll`` provides scalable event monitoring for large connection sets, standard Unix utilities, network daemons and portable POSIX programs rely on the lightweight, stateless ``poll()`` interface to inspect small groups of file descriptors without the overhead of creating persistent kernel epoll instances.

Through Milestone 52, Keira introduces:

1. **Century-Plus Syscall 106 (``SYS_POLL``)**: System call number ``SYS_POLL = 106`` defined across kernel tables, vector resolution routines and architecture system call headers.
2. **File Descriptor Polling Engine**: A high-efficiency readiness evaluator in ``crates/syscall/src/dispatcher/handlers/poll.rs`` supporting standard POSIX event bitmasks (``POLLIN``, ``POLLPRI``, ``POLLOUT``, ``POLLERR``, ``POLLHUP`` and ``POLLNVAL``). The engine evaluates readiness across anonymous pipes, UNIX domain sockets (``AF_UNIX``), network sockets (TCP/UDP), character devices and virtual filesystem inodes.
3. **Timed Polling Mechanics**: Support for non-blocking status probes (``timeout == 0``), indefinite waits (``timeout < 0``) and high-resolution millisecond timeouts (``timeout > 0``) calibrated with HPET hardware timers.
4. **Supervisor ``pollinfo`` Diagnostic Command**: Diagnostic inspection command in ``crates/shell`` displaying real-time multiplexing invocations, signal counts, hangup detections and timeout telemetry.
5. **Freestanding C Library ``<poll.h>`` & ``poll()``**: Integration of ``<poll.h>`` and ``<sys/poll.h>`` into Keira's freestanding C standard library, implementing ``poll(struct pollfd *fds, nfds_t nfds, int timeout)``.
6. **Standalone ``/bin/test_poll`` Utility**: Implementation of ``/bin/test_poll.elf`` and ``/bin/test_poll``, expanding Keira's standalone core utility suite to 43 binaries.
7. **ABI Compliance Test Suite Expansion (Test 38)**: Addition of Test 38 to ``/bin/test_abi.elf``, verifying pipe readiness transitions, negative descriptor ignoring, unopened descriptor ``POLLNVAL`` detection, timeout expiration and peer disconnect ``POLLHUP`` reporting.

2. Architecture & Design
========================

.. code-block:: text

   flowchart TD
       subgraph Userspace["Ring 3 Userspace (C Runtime & Utilities)"]
           TEST_ABI["ABI Suite: /bin/test_abi.elf (Test 38)"]
           TEST_POLL["Core Utility: /bin/test_poll.elf (43rd Binary)"]
           LIBC_POLL["libc.a: poll(struct pollfd *, nfds, timeout)"]
           HDR_POLL["<poll.h> & <sys/poll.h>"]
           TEST_ABI --> LIBC_POLL
           TEST_POLL --> LIBC_POLL
           LIBC_POLL --> HDR_POLL
       end

       subgraph SyscallBoundary["POSIX Syscall ABI Vector 106"]
           INT80["int 0x80 / syscall instruction"]
           ROUTER["Dispatcher Router: Vector 106"]
           VALIDATOR["User Memory Pointer Validation (validate_user_ptr)"]
           LIBC_POLL --> INT80
           INT80 --> ROUTER
           ROUTER --> VALIDATOR
       end

       subgraph KernelEngine["Ring 0 I/O Multiplexing Engine"]
           POLL_HANDLER["handle_poll(fds_ptr, nfds, timeout)"]
           READINESS["check_poll_readiness(task, item)"]
           TELEMETRY["POLL_TELEMETRY (Calls, Signals, Timeouts)"]
           VALIDATOR --> POLL_HANDLER
           POLL_HANDLER --> READINESS
           POLL_HANDLER --> TELEMETRY
       end

       subgraph Subsystems["Monitored Resource Subsystems"]
           PIPES["keira-ipc: Anonymous Pipes (PIPE_TABLE)"]
           UNIX["keira-net: AF_UNIX Domain Sockets"]
           NETSOCK["keira-net: TCP/UDP Sockets"]
           DEV["keira-io: TTY & Serial UART"]
           FS["keira-fs: VFS Inodes"]
           READINESS --> PIPES
           READINESS --> UNIX
           READINESS --> NETSOCK
           READINESS --> DEV
           READINESS --> FS
       end

       subgraph Observability["Supervisor Diagnostics"]
           TELEMETRY --> POLLINFO["Supervisor Command 'pollinfo' (keira-shell)"]
       end

2.1 Standard Structure & Bitmasks
---------------------------------

The POSIX ``struct pollfd`` structure layout is defined symmetrically in both C and Rust:

.. code-block:: c

   struct pollfd {
       int   fd;       /* File descriptor to inspect */
       short events;   /* Requested event bitmask */
       short revents;  /* Returned event bitmask */
   };

.. list-table::
   :header-rows: 1
   :widths: 10 10 39

   * - Constant
     - Value
     - Description
   * - ``POLLIN``
     - ``0x0001``
     - There is data to read without blocking
   * - ``POLLPRI``
     - ``0x0002``
     - There is urgent data to read
   * - ``POLLOUT``
     - ``0x0004``
     - Writing data will not block
   * - ``POLLERR``
     - ``0x0008``
     - Error condition occurred on descriptor
   * - ``POLLHUP``
     - ``0x0010``
     - Hangup occurred (peer closed channel)
   * - ``POLLNVAL``
     - ``0x0020``
     - Invalid request: descriptor is not open

2.2 Subsystem Readiness Inspection
----------------------------------

When evaluating a file descriptor:

1. **Negative Descriptors (``fd < 0``)**: As mandated by POSIX.1-2008, negative descriptor numbers are ignored and their ``revents`` field is set strictly to zero.
2. **Invalid Descriptors (``fd >= MAX_FDS`` or not open)**: The engine marks ``revents = POLLNVAL`` and increments telemetry counters.
3. **Anonymous Pipes (``is_pipe``)**: Polled via ``keira_ipc::pipe::query_pipe_readiness``. Reader endpoints report ``POLLIN`` if buffered bytes exist or if all writers closed. Writer endpoints report ``POLLOUT`` if available capacity remains and ``POLLERR`` if all readers disconnected. Closed writers trigger ``POLLHUP`` on reader endpoints.
4. **UNIX Domain & Network Sockets (``is_socket``)**: Polled via ``socket_is_readable`` and ``socket_is_writable``. Read readiness triggers when RX buffers contain payload data or when remote connections complete.
5. **Character Devices & VFS Inodes**: Standard streams (stdout, stderr) report ``POLLOUT``, while regular files report both ``POLLIN`` and ``POLLOUT``.

3. Observability & Userland Verification
========================================

3.1 Freestanding /bin/test_poll Utility Execution
-------------------------------------------------

.. code-block:: text

   keira:/# test_poll.elf
   ==================================================
   Keira Kernel POSIX I/O Multiplexing Verification
   ==================================================
   [TEST 1] Poll readiness transitions on anonymous pipe...
     [OK] Pipe readiness transitions verified
   [TEST 2] Negative and unopened descriptor handling...
     [OK] Negative descriptor ignoring and POLLNVAL verified
   [TEST 3] Millisecond timeout expiration on idle descriptor...
     [OK] Timeout expiration verified
   [TEST 4] Peer disconnect and POLLHUP propagation...
     [OK] POLLHUP detection verified
   ==================================================
   [DONE] All POSIX poll test cases PASSED.
   ==================================================

3.2 Master ABI Test Suite Expansion (38/38 Passed)
--------------------------------------------------

.. code-block:: text

   keira:/# test_abi.elf
   ...
   [TEST] UNIX domain socketpair (AF_UNIX & Syscall 105) bidirectional IPC...
   [OK]   UNIX domain socketpair (AF_UNIX & Syscall 105) bidirectional IPC verified
   [TEST] POSIX Synchronous I/O multiplexing & sys_poll (Syscall 106)...
   [OK]   POSIX Synchronous I/O multiplexing & sys_poll (Syscall 106) verified
   ...
   [DONE] All Ring 3 Syscall Security & Fault Injection tests PASSED.

3.3 Supervisor Ring 0 Diagnostic Command (pollinfo)
---------------------------------------------------

.. code-block:: text

   ring0@keira:/# pollinfo
   === POSIX I/O Multiplexing Subsystem (SYS_POLL) ===

     Total sys_poll Invocations    : 14 calls
     Total Ready Event Signals     : 18 events
     Timeout Expirations           : 2 events
     Hangup (POLLHUP) Events       : 2 events
     Invalid FD (POLLNVAL)         : 2 events
     Max Monitored Descriptors     : 64 descriptors / call

     Supported Channels            : Anonymous Pipes, UNIX Sockets, TCP/UDP, TTY & VFS
     Standard Compliance           : POSIX.1-2008 / Single UNIX Specification v4

4. Automated Testing & Verification
===================================

Milestone 52 achieved complete verification across all quality gates:

1. **Unit Tests**: All 12 crates passed unit testing with 0 failures (``cargo test --workspace -- --test-threads=1``).
2. **Dual-Architecture Toolchain**: Verified clean compilation for both ``x86_64`` and ``i686`` targets under ``#![deny(warnings)]`` (``make full``).
3. **Core Utility Suite**: Expanded core utility count to 43 standalone binaries with the addition of ``/bin/test_poll``.
4. **ABI Compliance Test Suite**: All 38 ABI tests passed with 100% compliance.
