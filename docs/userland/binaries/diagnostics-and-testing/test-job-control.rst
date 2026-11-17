.. SPDX-License-Identifier: GPL-2.0-only


test_job_control.elf Freestanding Core Utility
==============================================

The ``test_job_control.elf`` executable (``/bin/test_job_control.elf`` and ``/bin/test_job_control``) is a freestanding Ring 3 test utility validating POSIX process groups, session leader management and job control system calls (``SYS_SETPGID``, ``SYS_GETPGRP``, ``SYS_GETPGID`` and ``SYS_SETSID``).

1. Architectural Purpose
========================

In accordance with Milestone 57, process group isolation and session tracking verification is implemented as a freestanding unprivileged Ring 3 binary:

1. **Ring 3 Privilege Isolation**: Executes entirely in unprivileged Ring 3, invoking process group syscalls through standard libc wrappers.
2. **Process Group Identification**: Validates ``getpgrp()`` and ``getpgid()`` return values against calling process identifiers and handles invalid process boundaries.
3. **Process Group Mutation**: Asserts ``setpgid(0, 0)`` correctly assigns the target task's PGID to its PID.
4. **Child Group Isolation**: Validates that forked children inherit the parent's process group and can be successfully detached into an isolated process group.
5. **Session Creation**: Tests ``setsid()`` establishing a new session and process group and verifies the POSIX invariant rejecting subsequent ``setsid()`` calls from session leaders with ``EPERM``.

2. Test Battery Matrix
======================

.. list-table::
   :header-rows: 1
   :widths: 10 44 118

   * - Test Suite
     - Target Subsystem
     - Assertions
   * - **Test 1**
     - Process Group Retrieval
     - Verifies ``getpgrp()`` matches ``getpgid(0)`` and ``getpgid(self)``, rejecting invalid PIDs with ``EINVAL`` or ``ESRCH``
   * - **Test 2**
     - Calling Process Mutation
     - Validates ``setpgid(0, 0)`` assigning PGID to PID
   * - **Test 3**
     - Child Group Isolation
     - Verifies child inherits parent PGID, mutates to independent group and exits with code 42 reaped by parent
   * - **Test 4**
     - Session Creation & Leadership
     - Asserts ``setsid()`` establishes new session and process group, with subsequent calls failing with ``EPERM``
   * - **Test 5**
     - Parameter Bounds & Error Safety
     - Asserts invalid process identifiers return ``EINVAL`` or ``ESRCH`` without kernel faults

3. Usage Examples
=================

Executing Full Job Control Test Battery
---------------------------------------

.. code-block:: bash

   keira:/# test_job_control.elf
   ==================================================
   Keira Kernel POSIX Process Groups and Job Control
   ==================================================
   [TEST 1] Process group retrieval (getpgrp / getpgid)...
     Current PID: 3, PGID: 3
     [OK] Process group identification verified
   [TEST 2] Mutating calling process group (setpgid)...
     [OK] Self setpgid mutation verified
   [TEST 3] Child process group creation and isolation...
     [OK] Child process group isolation verified
   [TEST 4] Session creation and group leadership (setsid)...
     [OK] Session creation and group leadership verified
   [TEST 5] Parameter bounds and permission checks...
     [OK] Parameter boundary error handling verified
   ==================================================
   [DONE] All POSIX job control test cases PASSED.
   ==================================================
