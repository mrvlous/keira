.. SPDX-License-Identifier: GPL-2.0-only

==================================
Linux Kernel SPDX Licensing Policy
==================================

The Keira Kernel adheres strictly to the Linux Kernel SPDX licensing rules and the REUSE 3.0 specification. All source files within the repository must contain an explicit SPDX license expression on the first line.

1. SPDX License Identifier Format
=================================

The license expression must appear on the very first line of every source file using the following format:

.. code-block:: text

   SPDX-License-Identifier: <SPDX-License-Expression>

No verbose license preamble, copyright declaration or legal boilerplate may precede this tag.

2. Syntax by File Type
======================

The SPDX tag must use the language-appropriate comment syntax for the target file format:

.. list-table:: SPDX Comment Syntax Reference
   :widths: 20 40 40
   :header-rows: 1

   * - Language / File Type
     - Syntax
     - Example
   * - C / C++ Source & Headers
     - Single-line C++ comment
     - ``// SPDX-License-Identifier: GPL-2.0-only``
   * - Rust Crates & Modules
     - Single-line Rust comment
     - ``// SPDX-License-Identifier: GPL-2.0-only``
   * - Assembly (NASM / GAS)
     - Semicolon comment
     - ``; SPDX-License-Identifier: GPL-2.0-only``
   * - Python, Makefiles & Shell
     - Hash comment
     - ``# SPDX-License-Identifier: GPL-2.0-only``
   * - reStructuredText (``.rst``)
     - Sphinx comment
     - ``.. SPDX-License-Identifier: GPL-2.0-only``

3. Formatting Invariants
========================

1. **Line 1 Placement**: The SPDX directive must occupy line 1 of the file. No blank lines or comments may appear before line 1.
2. **Empty Line Following**: Line 2 must be an empty line separating the license tag from file contents.
3. **Valid Identifiers**: Only identifiers registered in the SPDX License List (e.g. ``GPL-2.0-only``, ``MIT``, ``BSD-2-Clause``) are permissible.
4. **Automated Enforcement**: All pull requests are verified via ``make check-licenses`` or automated CI linters to guarantee zero missing headers.
