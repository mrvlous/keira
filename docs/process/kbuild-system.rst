.. SPDX-License-Identifier: GPL-2.0-only

Linux-Grade Kbuild Architecture & Modular Build System
======================================================

The Keira Kernel incorporates a Linux-grade, modular Kbuild compilation system.
Operating in direct harmony with the Kconfig declarative configuration engine
(Milestone 59), Kbuild eliminates centralized hardcoded source lists by distributing
modular ``Kbuild`` manifests across architecture trees and userland package directories.

Overview & Architectural Philosophy
-----------------------------------

In traditional monolithic build configurations, source file paths are centralized in
a single master Makefile. As the codebase expands across multiple architectures,
subsystems and userland packages, centralized lists become fragile, repetitive and
prone to merge conflicts.

The upstream Linux Kernel resolved this scalability barrier through Kbuild: a declarative,
recursive compilation framework where each directory maintains a local ``Kbuild`` file
specifying target objects conditioned on configuration symbols (``obj-$(CONFIG_*) += ...``).

Keira embraces this exact architectural pattern:

1. **Subsystem Autonomy**: Each architectural subdirectory and userland package declares
   its compilation targets independently.
2. **Kconfig Binding**: Target inclusion maps directly to Kconfig symbols emitted in
   ``.config`` and ``include/generated/autoconf.h``.
3. **Core Primitives Inclusion**: The root orchestrator includes ``scripts/Kbuild.include``
   providing standard compiler probes and path normalization helpers.
4. **Dual-Architecture Parity**: Assembly sources for 64-bit Long Mode and 32-bit Protected
   Mode are decoupled into dedicated manifests.

Core Helper Primitives (scripts/Kbuild.include)
-----------------------------------------------

The shared file ``scripts/Kbuild.include`` establishes standard compiler probes and
path manipulation macros modeled after Linux kernel standards:

.. code-block:: makefile

   # Probe if assembler supports flag
   as-option = $(shell if $(ASM) $(1) -o /dev/null /dev/null >/dev/null 2>&1; \
       then echo "$(1)"; else echo "$(2)"; fi)

   # Probe if C compiler supports flag
   cc-option = $(shell if $(CC) $(1) -c -x c /dev/null -o /dev/null >/dev/null 2>&1; \
       then echo "$(1)"; else echo "$(2)"; fi)

   # Suppress compiler warning if supported
   cc-disable-warning = $(call cc-option,-Wno-$(strip $(1)))

   # Normalize directory paths by removing trailing slash
   strip-slash = $(patsubst %/,%,$(1))

   # Prefix objects with directory path
   add-prefix = $(addprefix $(strip $(1))/,$(2))

Hierarchical Kbuild Tree Organization
-------------------------------------

The Kbuild manifest tree mirrors subsystem boundaries across the kernel repository:

.. code-block:: text

   .
   ├── Makefile                    # Master build orchestrator (includes Kbuild.include)
   ├── arch/
   │   └── x86/
   │       ├── Kbuild              # Master x86 dispatcher
   │       ├── common/
   │       │   └── Kbuild          # Multiboot2 bootstrap assembly
   │       ├── i686/
   │       │   └── Kbuild          # 32-bit Protected Mode assembly
   │       └── x86_64/
   │           └── Kbuild          # 64-bit Long Mode assembly
   ├── crates/
   │   └── Kbuild                  # Modular kernel crate workspace manifest
   ├── scripts/
   │   └── Kbuild.include          # Standard Kbuild helper macros
   └── userland/
       ├── Makefile                # Userland build orchestrator
       ├── Kbuild                  # Master userland dispatcher
       ├── coreutils/Kbuild        # POSIX core utilities
       ├── init/Kbuild             # PID 1 init and power management
       ├── libc/Kbuild             # Freestanding C runtime library
       ├── netutils/Kbuild         # Network utilities and fetch
       ├── procutils/Kbuild        # Process inspection utilities
       ├── shell/Kbuild            # Standalone userspace shell
       ├── sysutils/Kbuild         # System administration and sysinfo
       ├── tests/Kbuild            # ABI test suites and fuzzing
       └── toolchain/Kbuild        # Native C compiler (kcc)

Architecture Assembly Manifests (arch/x86/)
-------------------------------------------

The top-level architecture manifest ``arch/x86/Kbuild`` dynamically delegates to
subdirectories based on configuration variables:

.. code-block:: makefile

   # Common Multiboot2 bootstrap assembly header
   include arch/x86/common/Kbuild

   # Architecture-specific bootstrap and kernel low-level assembly
   ifeq ($(CONFIG_ARCH_I686),y)
       include arch/x86/i686/Kbuild
   else
       include arch/x86/x86_64/Kbuild
   endif

When evaluated by the root ``Makefile``, the variable ``$(asm-src-y)`` aggregates
all active assembly sources, generating target object paths in ``$(OBJ_DIR)``
automatically.

Userland Package Manifests (userland/)
--------------------------------------

The userland build system utilizes standard Kbuild symbol resolution semantics:

* Symbols enabled with ``=y`` in ``.config`` evaluate directly to active target lists
  (e.g. ``bin-y`` and ``multi-bin-y``).
* Symbols disabled with ``=n`` or unset evaluate to inactive lists (e.g. ``bin-n``
  and ``multi-bin-n``), which are naturally ignored during linking.

Example from ``userland/tests/Kbuild``:

.. code-block:: makefile

   # ABI Verification & Diagnostic Test Suites Kbuild Manifest
   bin-$(CONFIG_USERLAND_TESTS)        += test_job_control \
                                          test_pipe \
                                          test_poll \
                                          test_select \
                                          test_threads

   multi-bin-$(CONFIG_USERLAND_TESTS)  += test_abi \
                                          fuzz_abi

Example from ``userland/toolchain/Kbuild``:

.. code-block:: makefile

   # Native C compiler toolchain bound to CONFIG_USERLAND_COMPILER_KCC
   multi-bin-$(CONFIG_USERLAND_COMPILER_KCC)    += kcc

Build System Integration Benefits
---------------------------------

1. **Elimination of Duplication**: Source lists are declared once in local ``Kbuild``
   files rather than duplicated across multiple Makefiles.
2. **Deterministic Configuration**: Changes in ``.config`` immediately take effect
   across all subdirectories without modifying build scripts.
3. **Purity of Makefiles**: ``Makefile`` and ``userland/Makefile`` focus strictly on
   compilation recipes, compiler flags and linker rules.
4. **Linux Developer Parity**: Developers familiar with the Linux kernel build system
   experience an identical workflow and syntax.
