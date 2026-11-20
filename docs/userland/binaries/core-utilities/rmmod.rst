.. SPDX-License-Identifier: GPL-2.0-only


rmmod
=====

The ``rmmod`` utility unloads an idle module from the running kernel via the ``sys_delete_module`` system call.

Synopsis
--------

.. code-block:: text

   rmmod <module_name>
   rmmod -h | --help

Options
-------

.. option:: -h, --help

   Display program synopsis, available options and exit.

Description
-----------

``rmmod`` verifies that the target module exists in ``MODULE_TABLE`` and has zero active references before requesting the kernel to unregister it via system call vector 35. If the module is currently locked by active consumers, the kernel rejects the deletion request with ``-EBUSY``.

Examples
--------

Unload an idle kernel module:

.. code-block:: console

   $ rmmod my_driver
   rmmod: module 'my_driver' unloaded successfully

Attempting to unload a module in use:

.. code-block:: console

   $ rmmod ext4_fs
   rmmod: ERROR: could not remove module 'ext4_fs': error -16
