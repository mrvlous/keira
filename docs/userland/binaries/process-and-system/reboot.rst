.. SPDX-License-Identifier: GPL-2.0-only


reboot.elf Freestanding Core Utility
====================================

The ``reboot.elf`` executable (``/bin/reboot.elf`` and ``/bin/reboot``) is a freestanding Ring 3 core utility that resets the processor and restarts the machine.

1. Architectural Purpose
========================

System restart operations adhere to standard POSIX conventions:

1. **Clean Disk Synchronization**: Automatically calls ``sync()`` prior to restarting, flushing cached disk blocks and page cache dirty buffers to persistent storage.
2. **Linux-Compatible System Call**: Invokes Century-Plus Syscall 109 (``SYS_REBOOT``) with magic numbers (``0xfee1dead``, ``672274793``) and command ``RB_AUTOBOOT`` (``0x01234567``).
3. **Privilege Demarcation**: Restricts execution strictly to root accounts (``UID 0``), preventing unprivileged disruption.

2. Command-Line Options
=======================

.. list-table::
   :header-rows: 1
   :widths: 10 11 46

   * - Option
     - Long Option
     - Description
   * - ``-f``
     - ``--force``
     - Force immediate reboot without calling sync
   * - ``-h``
     - ``--help``
     - Displays usage summary and exits with status 0

3. Usage Examples
=================

.. code-block:: bash

   # Restart the machine:
   keira:/# reboot

   # Force immediate restart without disk sync:
   keira:/# reboot -f

4. Technical Specifications
===========================

- **Source Path**: ``userland/apps/system/reboot.c``
- **Binary Locations**: ``/bin/reboot.elf``, ``/bin/reboot``
- **System Call**: Vector 109 (``SYS_REBOOT``)
- **C Header**: ``<sys/reboot.h>``
- **Privilege Level**: Ring 3 privileged (UID 0 required)
