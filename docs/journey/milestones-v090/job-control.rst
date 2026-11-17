.. SPDX-License-Identifier: GPL-2.0-only


Milestone 57: Process Groups and Canonical Job Control
======================================================

Milestone 57 introduces POSIX process groups, session leader management and canonical asynchronous shell job control (``&``, ``jobs``, ``fg`` and ``bg``) across both 64-bit and 32-bit targets.

1. Architectural Motivation
===========================

Modern UNIX operating systems manage groups of related processes using Process Group Identifiers (PGID) and Session Identifiers (SID). Prior to Milestone 57, each task possessed credentials (UID, EUID, GID and EGID) but lacked group and session identifiers. This prevented the standalone userspace shell (``sh.elf``) from tracking background jobs, organizing pipelines into distinct execution groups or bringing asynchronous tasks back to the foreground.

With Milestone 57:
1. **Task Control Block Tracking**: Every task maintains ``pgid: usize`` and ``sid: usize`` in its task control block.
2. **Inheritance & Isolation**: Forked tasks inherit their parent process group and session. Spawning new process groups isolates background jobs from receiving unwanted signals directed to the foreground terminal group.
3. **Four Canonical Syscalls**: Syscalls 111 (``SYS_SETPGID``), 112 (``SYS_GETPGRP``), 113 (``SYS_GETPGID``) and 114 (``SYS_SETSID``) provide full POSIX API parity for userspace runtimes.
4. **Shell Job Table & Builtins**: The canonical shell manages up to 32 simultaneous background tasks with non-blocking child status reaping and provides ``jobs``, ``fg`` and ``bg`` builtins.

2. System Call Registry Additions
=================================

.. list-table::
   :header-rows: 1
   :widths: 10 18 18 54

   * - Vector
     - Constant
     - POSIX Prototype
     - Functional Semantics
   * - ``111``
     - ``SYS_SETPGID``
     - ``int setpgid(pid_t pid, pid_t pgid)``
     - Sets the process group identifier of the specified task
   * - ``112``
     - ``SYS_GETPGRP``
     - ``pid_t getpgrp(void)``
     - Retrieves the process group identifier of the current task
   * - ``113``
     - ``SYS_GETPGID``
     - ``pid_t getpgid(pid_t pid)``
     - Retrieves the process group identifier of target task
   * - ``114``
     - ``SYS_SETSID``
     - ``pid_t setsid(void)``
     - Establishes a new session and process group for the calling task

3. Shell Execution Semantics
============================

Asynchronous Background Execution
---------------------------------

Appending ``&`` to any executable command instructs the shell to dispatch the task in a detached process group without blocking:

.. code-block:: bash

   keira:/# sleep 10 &
   [1] 4
   keira:/#

Listing Active Jobs
-------------------

The ``jobs`` builtin inspects the job table, reaps terminated children non-blockingly via ``WNOHANG`` and lists active tasks:

.. code-block:: bash

   keira:/# jobs
   [1]+  Running                 sleep 10 &

Foreground Job Promotion
------------------------

The ``fg`` builtin brings a background task into the foreground and waits synchronously for its completion:

.. code-block:: bash

   keira:/# fg 1
   sleep 10 &
   keira:/#

4. Dual-Architecture Verification
=================================

Verification is validated by ``test_job_control.elf``, asserting:
1. Retrieval of current process group and session identity.
2. Mutation of calling process group via ``setpgid(0, 0)``.
3. Child process group isolation across ``fork()`` boundaries.
4. Session creation via ``setsid()`` with group leadership invariant checks.
5. Parameter boundary error handling rejecting invalid process identifiers with ``EINVAL`` or ``ESRCH``.
