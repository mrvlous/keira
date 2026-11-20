.. SPDX-License-Identifier: GPL-2.0-only


test_modules
============

The ``test_modules`` diagnostic test runner verifies in-kernel module lifecycle management, system call interfaces and ProcFS telemetry conformance.

Synopsis
--------

.. code-block:: text

   test_modules

Description
-----------

``test_modules`` executes an automated regression suite verifying:

1. **ProcFS Modules Interface**: Validates that ``/proc/modules`` exists and correctly formats active baseline drivers (``ext4_fs``, ``e1000_nic`` and ``ahci_sata``).
2. **ProcFS Kernel Symbols**: Validates that ``/proc/kallsyms`` exports global kernel addresses and symbol names matching Linux conventions.
3. **Dynamic Module Registration**: Exercises system call vector 34 (``sys_init_module``) to insert a dynamic probe module into ``MODULE_TABLE``.
4. **Introspection Verification**: Confirms that newly inserted modules immediately appear in ``/proc/modules``.
5. **Dynamic Module Unloading**: Exercises system call vector 35 (``sys_delete_module``) to remove the probe module.
6. **Teardown Verification**: Confirms that unloaded modules are removed from ``/proc/modules``.

Return Value
------------

* Returns ``0`` if all six conformance tests pass successfully.
* Returns ``1`` upon any assertion failure or unexpected system call error code.
