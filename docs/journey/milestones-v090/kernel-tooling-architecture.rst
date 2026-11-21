.. SPDX-License-Identifier: GPL-2.0-only


Milestone 65: Linux-Grade Kernel Tooling Architecture (bloat-o-meter & faddr2line)
==================================================================================

Milestone 65 introduces Linux-grade developer introspection tooling and binary analysis infrastructure into Keira's ``scripts/`` directory. Following the activation of autonomous background worker daemons in Milestone 64 and dynamic relocatable modules in Milestone 63, this milestone provides essential utilities for binary footprint profiling (``scripts/bloat-o-meter``) and panic crash address symbolization (``scripts/faddr2line``).

1. Architectural Motivation & Zero-Bloat Invariants
===================================================

As a freestanding monolithic kernel in Ring 0, Keira enforces strict minimalism and efficiency across both modern 64-bit (``x86_64``) and pure 32-bit (``i686``) bare-metal targets:

1. **Deterministic Size Accounting**: In complex monolithic kernels, minor refactorings or compiler upgrades can introduce unintentional code bloat. A deterministic tool is required to audit symbol size changes across commits.
2. **Crash & Stack Trace Symbolization**: When kernel panics, page faults or hardware traps occur on bare metal, the CPU registers capture raw execution pointers (RIP/EIP). Developers require an instant mapping to exact source code files and lines without relying on interactive debugger sessions.
3. **Pure Python 3 Implementation**: Unlike Linux's historical Perl scripts, Keira's tooling is implemented entirely in modern, freestanding Python 3 without external package dependencies, ensuring instant execution across any POSIX developer workstation.

2. Binary Footprint Differential Profiler (``scripts/bloat-o-meter``)
=====================================================================

The ``scripts/bloat-o-meter`` utility computes per-symbol size differentials between two ELF binaries:

* **Symbol Table Parsing**: Executes ``nm --size-sort -C`` (or ``llvm-nm``) to extract allocated text, data and bss symbols along with exact byte sizes.
* **Rust Demangling**: Automatically parses demangled Rust and C symbol names, enabling clear attribution across modular crates.
* **Granular Breakdown**: Tracks newly added, removed, grown and shrunk symbols, computing total byte changes and percentage differentials.
* **Integration**: Accessible via command-line invocation or ``make bloat`` with ``PREV=<old>`` and ``CURR=<new>`` parameters.

3. Fault Address & Stack Frame Symbolizer (``scripts/faddr2line``)
==================================================================

The ``scripts/faddr2line`` utility translates execution addresses into source code locations:

* **Direct Virtual Addresses**: Resolves raw hexadecimal addresses (e.g. ``0x001014d7``) using ``llvm-addr2line`` or ``addr2line``.
* **Symbolic Expressions**: Resolves expressions in ``<symbol>+<offset>/<size>`` or ``<symbol>+<offset>`` format (e.g. ``ap_main+0x10/0x145``), locating the symbol's base address in the ELF symbol table and adding the offset.
* **Boundary Validation**: Alerts developers if an offset exceeds the symbol's recorded boundary or if specified sizes mismatch.
* **Integration**: Accessible via command-line invocation or ``make faddr2line ADDR=<spec>``.

4. Build System & Test Integration
==================================

Milestone 65 integrates developer tooling into the master Makefile:

.. code-block:: makefile

   bloat: $(KERNEL_BIN) ## Compare kernel binary symbol sizes (PREV=<old_bin> CURR=<new_bin>)
   	$(Q)python3 scripts/bloat-o-meter $(if $(PREV),$(PREV),$(KERNEL_BIN)) $(if $(CURR),$(CURR),$(KERNEL_BIN))

   faddr2line: $(KERNEL_BIN) ## Translate fault address to source location (ADDR=<addr|sym+offset>)
   	$(Q)python3 scripts/faddr2line $(if $(BIN),$(BIN),$(KERNEL_BIN)) $(ADDR)

Dedicated self-test modes (``scripts/bloat-o-meter --test`` and ``scripts/faddr2line --test``) verify symbol parsing, delta calculation and address resolution with zero external dependencies, while ``scripts/kconfig/tests/`` remains strictly dedicated to Kconfig grammar and Kbuild primitives.
