.. SPDX-License-Identifier: GPL-2.0-only

Milestone 60: Linux-Grade Kbuild Infrastructure
===============================================

Milestone 60 introduces a Linux-grade, modular Kbuild compilation architecture to
Keira Kernel. Complementing the declarative Kconfig engine established in Milestone 59,
Milestone 60 eliminates hardcoded source arrays from master Makefiles and establishes
hierarchical ``Kbuild`` manifests across architecture subtrees and userland packages.

Historical Context & The Build Scalability Barrier
--------------------------------------------------

Across 59 preceding milestones, Keira Kernel evolved into a full-featured SMP
operating system spanning 12 crates, 114 POSIX system calls, bare-metal networking
and 47 Ring 3 userland utilities. However, source files and compilation targets
were manually enumerated in root ``Makefile`` and ``userland/Makefile``.

While functional for early milestones, centralized source arrays created distinct
architectural drawbacks:

1. **Tight Coupling**: Modifying an architecture assembly routine or adding a new
   userland utility required editing top-level Makefiles.
2. **Conditional Complexity**: Selective feature toggles (such as disabling ``kcc``
   or ABI test suites) relied on manual ``filter-out`` macro logic.
3. **Subsystem Isolation**: Subdirectories lacked self-contained build manifests,
   deviating from the modular organization found in production operating systems.

Architectural Design & Subdirectory Kbuild Manifests
----------------------------------------------------

Milestone 60 resolves these challenges by introducing standard Linux kernel Kbuild
principles to Keira:

1. **Root Aggregator & Helpers**: Implements root ``Kbuild`` manifest and
   ``scripts/Kbuild.include`` standard option probes (``as-option``, ``cc-option``,
   ``cc-disable-warning``) and path normalization helpers.
2. **Architecture Separation** (``arch/x86/Kbuild``): Decouples x86 bootstrap
   into ``arch/x86/common/Kbuild``, ``arch/x86/x86_64/Kbuild`` and
   ``arch/x86/i686/Kbuild``.
3. **Userland Package Autonomy** (``userland/Kbuild``): Distributes utility targets
   across dedicated manifests:
   * ``userland/coreutils/Kbuild``: Foundational POSIX tools.
   * ``userland/procutils/Kbuild``: Process inspection and telemetry.
   * ``userland/sysutils/Kbuild``: System administrative commands and sysinfo.
   * ``userland/init/Kbuild``: PID 1 init and shutdown management.
   * ``userland/shell/Kbuild``: Standalone userspace shell.
   * ``userland/netutils/Kbuild``: Network configuration and fetch utilities.
   * ``userland/toolchain/Kbuild``: Native C compiler (``kcc``).
   * ``userland/tests/Kbuild``: Diagnostic test suites and ABI fuzzers.
   * ``userland/libc/Kbuild``: Freestanding C runtime library.

Kconfig-to-Kbuild Symbol Binding
--------------------------------

Kbuild files leverage declarative symbol bindings directly tied to ``.config``:

* Enabled options (``CONFIG_FOO=y``) expand ``bin-$(CONFIG_FOO)`` to ``bin-y``.
* Disabled options (``CONFIG_FOO=n``) expand to ``bin-n``, leaving inactive
  targets unbuilt without manual conditionals.

Dual-Architecture Verification & Results
----------------------------------------

The Kbuild infrastructure was verified through extensive testing:

1. **Dual-Architecture Parity**: Both 64-bit (``x86_64``) and 32-bit (``i686``)
   targets compile, package and boot cleanly in QEMU.
2. **Clean Make Orchestration**: Master ``Makefile`` and ``userland/Makefile``
   dynamically resolve 47 userland binaries and architecture assembly sources.
3. **Toolchain Consistency**: Enforced 8-column hardware tabs for all ``Kbuild*``
   manifests via ``.editorconfig`` and added tracking in ``MAINTAINERS``.
4. **Code Quality**: Validated 100% compliance via ``scripts/checkpatch.py`` and
   ``make docs-lint``.
