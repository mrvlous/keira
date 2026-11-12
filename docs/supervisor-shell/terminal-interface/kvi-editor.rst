.. SPDX-License-Identifier: GPL-2.0-only


General Utility Commands
========================

The ``util`` command suite provides emergency diagnostics assistance for the supervisor console. Terminal screen clearing in Ring 0 is handled via the native supervisor command ``wipe``.

Command Reference
=================

.. list-table::
   :header-rows: 1
   :widths: 10 10 60 15

   * - Command
     - Syntax
     - Description
     - Flags / Options
   * - ``help``
     - ``help``
     - Display interactive emergency command index or command usage
     - ``-h, --help``
