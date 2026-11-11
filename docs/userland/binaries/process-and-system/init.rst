.. SPDX-License-Identifier: GPL-2.0-only

================================================
init.elf Canonical Userspace Init Binary (PID 1)
================================================

The canonical Ring 3 init executable (``/bin/init.elf`` and ``/bin/init``) is the first userland process spawned by the Keira kernel upon completing hardware initialization and mounting the root Virtual Filesystem (VFS).

1. Architectural Purpose
========================

Keira is designed as a freestanding monolithic kernel. Under this architecture, the kernel isolates hardware abstraction, memory paging, task scheduling and interrupt handling inside Ring 0, delegating early userspace coordination to Ring 3 PID 1:

1. **Ring 3 Entry Verification**: Validates that privilege level transitions (via ``sysret``, ``sysexit`` or ``iret``) succeed cleanly from Ring 0 to Ring 3.
2. **Standard POSIX Execution**: Runs as a standard static ELF executable compiled with ``<stdio.h>``, ``<unistd.h>`` and ``<sys/wait.h>`` from Keira's freestanding libc.
3. **Canonical System Bootstrap**: Detects and executes ``/etc/init.sh`` via ``/bin/sh`` to configure hostname, mount in-memory TmpFS on ``/tmp``, set POSIX DAC sticky-bit permissions (``chmod 1777 /tmp``), create standard runtime directories (``/var/run``, ``/var/log``) and inspect network interfaces.
4. **Graceful Handover**: Completes userspace system orchestration and exits cleanly with status 0, allowing the kernel runtime to launch the interactive shell session (``/bin/sh.elf``) or drop into supervisor mode.

2. Invocation & Shell Integration
=================================

While automatically executed at boot time as PID 1, ``init.elf`` can also be inspected and re-executed dynamically:

.. code-block:: bash

   # Display help and usage reference:
   keira:/# /bin/init.elf -h
   Usage: init [OPTIONS]

   Description:
     Canonical userspace init system (PID 1).

   Options:
     -v, --verbose  Display detailed initialization status
     -h, --help     Display this help reference and exit

   # Standard execution (spawns /etc/init.sh system bootstrap):
   keira:/# /bin/init.elf
   [INIT] Keira Canonical Userspace Init (PID 1) active in Ring 3
   [INIT] Executing canonical system bootstrap: /etc/init.sh
   [INIT] Initializing canonical userland system environment...
   mount: successfully mounted none on /tmp (type tmpfs)
   lo: flags=73<UP,LOOPBACK,RUNNING>  mtu 65536
           inet 127.0.0.1  netmask 255.0.0.0
   Keira keira 0.8.0 #1 SMP 2026 x86_64
   [INIT] Userland system initialization sequence completed.
   [INIT] System bootstrap sequence complete.
