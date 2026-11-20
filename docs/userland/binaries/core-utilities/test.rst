.. SPDX-License-Identifier: GPL-2.0-only


test.elf Freestanding Core Utility
==================================

The ``test.elf`` executable (``/bin/test.elf``, ``/bin/test`` and ``/bin/[``) is a freestanding Ring 3 core utility that evaluates conditional expressions and returns exit status 0 (true) or 1 (false).

1. Architectural Purpose
========================

As part of Keira Milestone 61 (POSIX Userland Shell Scripting Language Engine), ``test.elf`` provides standardized condition testing for script branching and pipeline automation:

1. **Ring 3 Privilege Isolation**: Executes entirely in unprivileged userspace, performing file status checks via ``sys_stat`` and permission checks via ``sys_access``.
2. **Dual-Invocation Semantics**: Supports both standard ``test EXPRESSION`` and bracket syntax ``[ EXPRESSION ]`` where the closing bracket is verified.
3. **Comprehensive Expression Operators**: Evaluates file types, attributes, string equality, integer comparisons and logical operators without external runtime libraries.

2. Supported Operators
======================

File Operators
--------------

.. list-table::
   :header-rows: 1
   :widths: 15 65

   * - Operator
     - Description
   * - ``-e FILE``
     - True if file exists
   * - ``-f FILE``
     - True if file exists and is a regular file
   * - ``-d FILE``
     - True if file exists and is a directory
   * - ``-s FILE``
     - True if file exists and has size greater than zero
   * - ``-r FILE``
     - True if file exists and read permission is granted
   * - ``-w FILE``
     - True if file exists and write permission is granted
   * - ``-x FILE``
     - True if file exists and execute permission is granted

String Operators
----------------

.. list-table::
   :header-rows: 1
   :widths: 25 55

   * - Operator
     - Description
   * - ``-z STRING``
     - True if string length is zero
   * - ``-n STRING``
     - True if string length is non-zero
   * - ``STR1 = STR2``
     - True if strings are identical
   * - ``STR1 != STR2``
     - True if strings are not identical

Integer Comparisons
-------------------

.. list-table::
   :header-rows: 1
   :widths: 25 55

   * - Operator
     - Description
   * - ``INT1 -eq INT2``
     - True if integers are equal
   * - ``INT1 -ne INT2``
     - True if integers are not equal
   * - ``INT1 -lt INT2``
     - True if INT1 is less than INT2
   * - ``INT1 -le INT2``
     - True if INT1 is less than or equal to INT2
   * - ``INT1 -gt INT2``
     - True if INT1 is greater than INT2
   * - ``INT1 -ge INT2``
     - True if INT1 is greater than or equal to INT2

Logical Operators
-----------------

.. list-table::
   :header-rows: 1
   :widths: 25 55

   * - Operator
     - Description
   * - ``! EXPR``
     - Invert expression truth value
   * - ``EX1 -a EX2``
     - True if both expressions are true
   * - ``EX1 -o EX2``
     - True if either expression is true

3. Usage Examples
=================

File Testing
------------

.. code-block:: bash

   # Verify file existence and type:
   keira:/# test -f /etc/hostname && echo "File exists"
   File exists

   # Bracket syntax:
   keira:/# [ -d /bin ] && echo "Directory verified"
   Directory verified

String & Integer Testing
------------------------

.. code-block:: bash

   # String equality check:
   keira:/# [ "keira" = "keira" ] && echo "Match"
   Match

   # Integer comparison:
   keira:/# [ 10 -gt 5 ] && echo "Greater"
   Greater
