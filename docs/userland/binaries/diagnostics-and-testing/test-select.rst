.. SPDX-License-Identifier: GPL-2.0-only


test_select.elf Freestanding Core Utility
=========================================

The ``test_select.elf`` executable (``/bin/test_select.elf`` and ``/bin/test_select``) is a freestanding Ring 3 test utility validating the POSIX ``select`` synchronous I/O multiplexing subsystem and Century-Plus Syscall 107 (``SYS_SELECT``).

1. Architectural Purpose
========================

In accordance with UNIX purity and the Keira v0.8.0 milestone roadmap, synchronous select multiplexing verification is implemented as a freestanding unprivileged Ring 3 binary:

1. **Ring 3 Privilege Isolation**: Executes entirely in unprivileged Ring 3, invoking Syscall 107 (``SYS_SELECT``) through the libc wrapper ``select()``.
2. **Bitmask Manipulation Verification**: Validates canonical ``fd_set`` bitmask operations (``FD_ZERO``, ``FD_SET``, ``FD_CLR``, ``FD_ISSET``) across read, write and exception descriptor sets.
3. **Multi-Descriptor Routing**: Verifies accurate event demultiplexing across multiple open pipe channels and file descriptor boundaries.
4. **Non-Blocking & Timeout Semantics**: Validates zero-timeout polling with ``struct timeval`` structures and immediate non-blocking returns.

2. Test Battery Matrix
======================

.. list-table::
   :header-rows: 1
   :widths: 10 44 118

   * - Test Suite
     - Target Subsystem
     - Assertions
   * - **Test 1**
     - Pipe Readiness Transitions
     - Verifies writable readiness on write end and non-readable status on empty read end, followed by readability upon write
   * - **Test 2**
     - Multi-Descriptor Bitmask Multiplexing
     - Tests concurrent multiplexing across multiple distinct pipes and validates bitmask filtering
   * - **Test 3**
     - Zero-Timeout Non-Blocking
     - Asserts immediate return without sleeping when ``struct timeval`` is configured to 0 seconds and 0 microseconds
   * - **Test 4**
     - Closed Pipe Handoff (``EOF``)
     - Verifies that closing a pipe write end correctly flags readability to allow child processes to detect EOF
   * - **Test 5**
     - Descriptor Boundary & Error Safety (``EBADF``)
     - Asserts that invalid descriptors or negative descriptor bounds return ``EBADF`` without kernel faults

3. Usage Examples
=================

Executing Full Select Test Battery
----------------------------------

.. code-block:: bash

   keira:/# test_select.elf
   [TEST 1] Select readiness transitions on anonymous pipe...
     [PASS] Empty pipe correctly reported writable and not readable
     [PASS] Pipe reader correctly reported readable after write
     [PASS] Drain read succeeded
   [TEST 2] Multi-descriptor bitmask multiplexing...
     [PASS] Multiplexed select correctly detected only pipe B readable
   [TEST 3] Zero timeout non-blocking select...
     [PASS] Zero-timeout select on empty pipe returned 0 immediately
   [TEST 4] EOF and hangup handling on closed pipe writer...
     [PASS] Closed writer pipe correctly reported readable for EOF detection
   [TEST 5] Invalid file descriptor and boundary rejection (EBADF)...
     [PASS] Invalid descriptor correctly rejected with EBADF
   [SUMMARY] All 5 select() tests PASSED!

Executing Via Canonical Symlink
-------------------------------

.. code-block:: bash

   keira:/# test_select

4. Technical Specifications
===========================

- **Source Path**: ``userland/tests/test_select.c``
- **Binary Locations**: ``/bin/test_select.elf``, ``/bin/test_select``
- **Privilege Level**: Ring 3 unprivileged
- **Dependencies**: Freestanding ``libc.a`` (``<sys/select.h>``, ``<unistd.h>``, ``<stdio.h>``, ``<stdlib.h>``, ``<string.h>``, ``<errno.h>``)
- **System Calls**: ``sys_select`` (Vector 107), ``sys_pipe``, ``sys_read``, ``sys_write``, ``sys_close`` and ``sys_exit``
