.. SPDX-License-Identifier: GPL-2.0-only

Milestone 59: Linux-Grade Kconfig Architecture
==============================================

Milestone 59 introduces a Linux-grade configuration infrastructure to Keira
Kernel. Moving beyond static environment overrides, Keira establishes an
autonomous Kconfig system with hierarchical subsystem specifications, curses
interactive visual configuration (``make menuconfig``), batch profile loading
and automatic C header macro generation.

Historical Context & The Configuration Challenge
------------------------------------------------

As Keira matured across 58 milestones, the kernel grew from a minimal real-mode
bootloader to an SMP monolithic kernel with 12 crates, 114 POSIX syscalls,
network stacks, storage filesystems and 47 userland utilities. Managing this
expanding parameter space using Makefile variables and manual configuration
templates became increasingly complex.

In the Linux kernel, the Kconfig language and tooling provide an expressive,
modular mechanism for defining configuration choices, dependencies and defaults.
For Keira to maintain parity with production kernel architectures, adopting
standard Kconfig workflows was the natural progression.

Architectural Design & Engine Implementation
--------------------------------------------

The implementation of Milestone 59 centered on creating a self-contained Kconfig
engine in ``scripts/kconfig/`` that operates without external third-party
package dependencies:

1. **Parser & Grammar Model** (``scripts/kconfig/kconfig.py``): Implements a
   recursive file parser supporting ``mainmenu``, ``source``, ``menu``, ``endmenu``,
   ``choice``, ``endchoice``, ``config``, ``bool``, ``int``, ``string``,
   ``hex``, ``default``, ``depends on``, ``select`` and ``help`` directives.
2. **Interactive Terminal TUI** (``scripts/kconfig/mconf.py``): Utilizes the
   built-in Python ``curses`` library to provide a full-screen menuconfig
   experience with keyboard navigation, submenu drilldown and instant saving.
3. **Batch Processor** (``scripts/kconfig/conf.py``): Supports ``--defconfig``,
   ``--pure-defconfig``, ``--savedefconfig`` and ``--oldconfig`` for command-line
   builds and automated CI environments.
4. **Header Generator** (``scripts/kconfig/genheader.py``): Translates active
   symbols from ``.config`` into ``include/generated/autoconf.h`` with macro
   guards for C and assembly consumers.

Hierarchical Kconfig Subsystem Tree
-----------------------------------

To mirror the Linux kernel layout, configuration manifests are distributed
across subsystem directories rather than concentrated in a monolithic file:

* Root ``Kconfig``: Declares kernel mainmenu and sources architecture, crate
  and userland manifests.
* Architecture ``arch/x86/Kconfig``: Encapsulates CPU architecture selection
  (x86_64 vs i686), SMP multi-core toggles and maximum core counts.
* Core Subsystems ``crates/Kconfig``: Sources modular crate manifests:
  * ``crates/mem/Kconfig``: ASLR and W^X memory mitigations.
  * ``crates/task/Kconfig``: Work-stealing scheduler and io_uring engine.
  * ``crates/fs/Kconfig``: FAT12/16/32, EXT4 and dynamic procfs.
  * ``crates/net/Kconfig``: TCP/IP stack, BSD sockets and Intel e1000 driver.
  * ``crates/security/Kconfig``: RLIMIT sandboxing and cryptographic entropy.
* Userspace ``userland/Kconfig``: Ring 3 userland packaging, native C compiler
  (``kcc``) and ABI security test suites.

Dual-Architecture Defconfig Profiles
------------------------------------

Standard baseline defconfigs are established in ``arch/x86/configs/``:

* ``arch/x86/configs/x86_64_defconfig``: Production 64-bit kernel with SMP,
  virtual memory mitigations and complete userland utility suite.
* ``arch/x86/configs/i686_defconfig``: Pure 32-bit protected mode kernel with
  PAE paging, dual-core SMP and full userland parity.
* ``arch/x86/configs/pure_defconfig``: Freestanding Ring 0 supervisor console
  build with userland staging disabled.

Multi-Architecture Verification & Results
-----------------------------------------

The Kconfig architecture was validated through comprehensive verification:

1. **Defconfig Generation**: Successfully generated ``.config`` and matching
   ``include/generated/autoconf.h`` for ``x86_64``, ``i686`` and pure supervisor
   profiles.
2. **Savedefconfig Verification**: Verified minimal diff extraction against
   baseline defaults.
3. **Build Integration**: Both 64-bit and 32-bit kernel ISOs compile and link
   without errors or warnings using Kconfig-driven variables.
4. **Code Quality**: Validated using ``scripts/checkpatch.py`` and verified
   maintainer mapping via ``scripts/get_maintainer.py``.
