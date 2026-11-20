.. SPDX-License-Identifier: GPL-2.0-only


lsmod
=====

The ``lsmod`` binary reports the status of loaded kernel modules by formatting the contents of ``/proc/modules`` into a standardized tabular view.

Synopsis
--------

.. code-block:: text

   lsmod [OPTIONS]

Options
-------

.. option:: -h, --help

   Display program synopsis, available options and exit.

Description
-----------

``lsmod`` opens and reads the dynamic kernel introspection file ``/proc/modules``. It parses each entry and renders a formatted table showing the module name, memory footprint in bytes, active reference count, lifecycle state and loaded virtual address.

Examples
--------

List active kernel modules:

.. code-block:: console

   $ lsmod
   Module                  Size  Used by  State     Address
   ext4_fs                65536        1  Live      0xffff800000400000
   e1000_nic              32768        0  Live      0xffff800000410000
   ahci_sata              24576        2  Live      0xffff800000420000
