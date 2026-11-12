.. SPDX-License-Identifier: GPL-2.0-only


System Initialization & PID 1
=============================

Describes the transition from kernel boot to userland initialization.

Initialization Sequence
=======================

1. The kernel finishes driver bringup, mounts the root filesystem (``/``) and initializes the process table.
2. The kernel spawns the first userland process (PID 1) executing ``/bin/init.elf`` in Ring 3.
3. PID 1 executes ``/etc/init.sh`` via ``/bin/sh`` to orchestrate userspace configuration:

   - Configures the system hostname (``/bin/hostname``).
   - Mounts dynamic in-memory scratch volume on ``/tmp`` (``mount -t tmpfs none /tmp``).
   - Sets POSIX DAC sticky-bit permissions (``chmod 1777 /tmp``).
   - Creates standard runtime hierarchies (``/var/run``, ``/var/log``).
   - Inspects loopback and network interfaces (``ifconfig lo``).
   - Displays system banner and telemetry (``uname -a``).

4. Upon successful completion of ``/etc/init.sh``, PID 1 terminates cleanly, allowing the kernel to launch the canonical Ring 3 interactive shell (``/bin/sh.elf``).
5. In pure kernel mode (``USERLAND=0`` or ``make run-bare``), the kernel bypasses userland init and drops immediately into the Ring 0 emergency supervisor console (``keira:/#`` ).
6. Orphaned processes in userland are re-parented and reaped via ``waitpid()``.
