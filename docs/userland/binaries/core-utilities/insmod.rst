.. SPDX-License-Identifier: GPL-2.0-only


insmod
======

The ``insmod`` utility inserts a loadable module into the running kernel via the ``sys_init_module`` system call.

Synopsis
--------

.. code-block:: text

   insmod <module_name> [size]
   insmod -h | --help

Options
-------

.. option:: -h, --help

   Display program synopsis, available options and exit.

Description
-----------

``insmod`` requests the kernel to dynamically register and load a new module into the kernel's active module table (``MODULE_TABLE``). It transmits the target module identifier and allocated footprint to the kernel dispatcher via syscall vector 34.

Examples
--------

Insert a new module with default 32 KB footprint:

.. code-block:: console

   $ insmod my_driver
   insmod: module 'my_driver' inserted successfully at slot 3

Insert a module specifying custom memory size:

.. code-block:: console

   $ insmod virtio_net 65536
   insmod: module 'virtio_net' inserted successfully at slot 4
