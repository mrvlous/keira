.. SPDX-License-Identifier: GPL-2.0-only


poweroff.elf Freestanding Core Utility
======================================

The ``poweroff.elf`` executable (``/bin/poweroff.elf`` and ``/bin/poweroff``) is a freestanding Ring 3 core utility that initiates ACPI power down and shuts down the computer.

1. Architectural Purpose
========================

Machine poweroff operations adhere to standard POSIX conventions:

1. **Clean Disk Synchronization**: Automatically calls ``sync()`` prior to powering down, flushing cached disk blocks and page cache dirty buffers to persistent storage.
2. **Linux-Compatible System Call**: Invokes Century-Plus Syscall 109 (``SYS_REBOOT``) with magic numbers (``0xfee1dead``, ``672274793``) and command ``RB_POWER_OFF`` (``0x4321fedc``).
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
     - Force immediate poweroff without calling sync
   * - ``-h``
     - ``--help``
     - Displays usage summary and exits with status 0

3. Usage Examples
=================

.. code-block:: bash

   # Power off the machine:
   keira:/# poweroff

   # Force immediate poweroff without disk sync:
   keira:/# poweroff -f

4. Technical Specifications
===========================

- **Source Path**: ``userland/init/poweroff.c``
- **Binary Locations**: ``/bin/poweroff.elf``, ``/bin/poweroff``
- **System Call**: Vector 109 (``SYS_REBOOT``)
- **C Header**: ``<sys/reboot.h>``
- **Privilege Level**: Ring 3 privileged (UID 0 required)
