.. SPDX-License-Identifier: GPL-2.0-only


Kernel Developer Tooling Infrastructure
=======================================

Keira provides deterministic developer tooling in the ``scripts/`` directory modeled after the Linux kernel, implemented in pure Python 3 without external package dependencies.

1. Binary Footprint Profiler (``scripts/bloat-o-meter``)
========================================================

The ``bloat-o-meter`` utility computes symbol-level size differentials between two ELF binaries or symbol dumps:

* **Zero-Bloat Enforcement**: Continuously monitors whether code modifications inadvertently increase kernel binary footprint.
* **Symbol Breakdown**: Categorizes symbols into added, removed, grown and shrunk categories.
* **Demangled Identifiers**: Automatically demangles Rust and C symbol names for clear attribution across modules.

Command Syntax
--------------

.. code-block:: bash

   # Compare baseline and modified kernel binaries:
   python3 scripts/bloat-o-meter build/x86/x86_64/bin/keira.old build/x86/x86_64/bin/keira.bin

   # Via Makefile integration:
   make bloat PREV=build/x86/x86_64/bin/keira.old CURR=build/x86/x86_64/bin/keira.bin

   # Print compact summary only:
   python3 scripts/bloat-o-meter -c prev.bin next.bin

2. Fault Address Symbolizer (``scripts/faddr2line``)
====================================================

The ``faddr2line`` utility translates raw CPU instruction pointers (RIP/EIP) from panic messages, page fault traps and stack traces into exact source code filenames and line numbers:

* **Direct Address Translation**: Resolves hexadecimal virtual addresses (e.g. ``0x001014d7``).
* **Symbolic Offset Translation**: Resolves symbol-relative expressions (e.g. ``kswapd_step+0x18/0x80``).
* **Boundary Validation**: Validates that specified offsets do not exceed function boundary limits.

Command Syntax
--------------

.. code-block:: bash

   # Symbolize raw hex address:
   python3 scripts/faddr2line build/x86/x86_64/bin/keira.bin 0x001014d7

   # Symbolize symbol relative offset:
   python3 scripts/faddr2line build/x86/x86_64/bin/keira.bin ap_main+0x10/0x145

   # Via Makefile integration:
   make faddr2line ADDR=0x001014d7

3. Built-In Verification & Self-Testing
=======================================

Both utilities include self-test suites accessible via the ``--test`` flag:

.. code-block:: bash

   # Verify bloat-o-meter differential logic:
   python3 scripts/bloat-o-meter --test

   # Verify faddr2line address parsing logic:
   python3 scripts/faddr2line --test
