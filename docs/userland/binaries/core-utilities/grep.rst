.. SPDX-License-Identifier: GPL-2.0-only


grep.elf Freestanding Core Utility
==================================

The ``grep.elf`` executable (``/bin/grep.elf`` and ``/bin/grep``) is a freestanding Ring 3 text-matching utility that searches lines matching a substring pattern across input files or standard input streams.

1. Architectural Purpose
========================

As part of Keira Milestone 37 (Text Processing Pipeline Suite), ``grep.elf`` enables deterministic text filtering across pipelines and files:

1. **Ring 3 Privilege Isolation**: Executes entirely in unprivileged userspace, reading stream data through POSIX descriptors and printing matching lines to stdout.
2. **Stream & Multi-File Support**: Reads standard input when no file operands are specified and automatically prefixes filenames when searching multiple files.
3. **Matching Modes**: Supports case-insensitive search (``-i``), inverted matching (``-v``), 1-based line numbers (``-n``), match counts (``-c``) and matching file lists (``-l``).

2. Command-Line Options
=======================

.. list-table::
   :header-rows: 1
   :widths: 12 68

   * - Option
     - Description
   * - ``-i``
     - Ignore case distinctions in patterns and input lines
   * - ``-v``
     - Invert matching logic to emit non-matching lines
   * - ``-n``
     - Prefix each matching output line with its 1-based line number
   * - ``-c``
     - Suppress matching lines and print only the total match count
   * - ``-l``
     - Suppress matching lines and print only filenames containing matches
   * - ``-h, --help``
     - Display usage summary and exit with status 0

3. Usage Examples
=================

Filtering Files and Pipeline Output
-----------------------------------

.. code-block:: bash

   # Search case-insensitively with line numbers:
   keira:/# grep -in localhost /etc/hosts
   1:127.0.0.1	localhost

   # Filter live telemetry through a shell pipeline:
   keira:/# cat /proc/meminfo | grep MemTotal
   MemTotal:         129804 kB

4. Technical Specifications
===========================

- **Source Path**: ``userland/coreutils/grep.c``
- **Binary Locations**: ``/bin/grep.elf``, ``/bin/grep``
- **Privilege Level**: Ring 3 unprivileged
- **Supported Architectures**: ``x86_64`` (64-bit Long Mode), ``i686`` (32-bit Protected Mode)
- **Standard Library Dependency**: ``libc.a`` (freestanding minimal C runtime)
