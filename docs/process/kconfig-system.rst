.. SPDX-License-Identifier: GPL-2.0-only

Linux-Grade Kconfig Architecture & Configuration System
=======================================================

The Keira Kernel incorporates a freestanding, Linux-grade Kconfig configuration
engine implemented in pure Python with standard curses terminal emulation. This
architecture provides developers with the familiar Linux kernel configuration
experience, including interactive visual menus (``make menuconfig``), batch
defconfig loading, minimal defconfig generation (``make savedefconfig``) and
automatic generation of C header macros in ``include/generated/autoconf.h``.

Overview & Architectural Philosophy
-----------------------------------

Operating system kernels manage diverse hardware platforms, optional subsystems
and compile-time features. Hardcoding these selections or relying solely on
scattered environment variables creates fragile build configurations. The Linux
kernel solved this challenge through Kconfig: a declarative domain-specific
language defining configuration symbols, hierarchical menus, dependency graphs
and default values.

Keira adopts this exact paradigm while respecting the boundary between safe Rust
and freestanding C userland:

1. **Declarative Specification**: Every major subsystem defines its configurable
   features in local ``Kconfig`` files located within its directory hierarchy.
2. **Unified Frontend**: A pure Python curses TUI (``scripts/kconfig/mconf.py``)
   provides an interactive menu interface with zero external pip dependencies.
3. **Reproducible Defconfigs**: Standard baseline configurations reside in
   ``arch/x86/configs/`` for 64-bit (``x86_64_defconfig``), 32-bit
   (``i686_defconfig``) and pure Ring 0 supervisor builds (``pure_defconfig``).
4. **Dual Output Generation**: Loading or saving configurations writes both the
   standard ``.config`` key-value file for Makefiles and a C preprocessor header
   ``include/generated/autoconf.h`` for userland and UAPI consumers.

Hierarchical Kconfig Tree Organization
--------------------------------------

The configuration tree is structured hierarchically, mirroring Linux kernel
subsystem layout:

.. code-block:: text

   .
   ├── Kconfig                     # Root configuration manifest
   ├── arch/
   │   └── x86/
   │       ├── Kconfig             # CPU architecture, SMP and APIC topology
   │       └── configs/            # Canonical baseline defconfigs
   │           ├── x86_64_defconfig
   │           ├── i686_defconfig
   │           └── pure_defconfig
   ├── crates/
   │   ├── Kconfig                 # Modular kernel crate index
   │   ├── fs/Kconfig              # FAT12/16/32, EXT4 and procfs storage
   │   ├── mem/Kconfig             # ASLR, W^X and virtual memory mitigations
   │   ├── net/Kconfig             # TCP/IP, BSD sockets and Intel e1000 driver
   │   ├── security/Kconfig        # RLIMIT and cryptographic entropy pools
   │   └── task/Kconfig            # Work-stealing scheduler and io_uring
   ├── scripts/
   │   └── kconfig/                # Autonomous Kconfig engine
   │       ├── conf.py             # Batch configuration processor
   │       ├── genheader.py        # Autoconf C header generator
   │       ├── kconfig.py          # Grammar parser and evaluator model
   │       └── mconf.py            # Curses interactive visual menuconfig
   └── userland/
       └── Kconfig                 # Ring 3 userland, compiler and test suites

Interactive Menuconfig Frontend (mconf.py)
------------------------------------------

The interactive frontend is invoked via:

.. code-block:: bash

   make menuconfig

The TUI is implemented in ``scripts/kconfig/mconf.py`` using the Python standard
library ``curses`` module. It presents developers with an intuitive navigable
hierarchy:

* **Arrow Keys (Up / Down)**: Navigate through menu items and options.
* **Enter**: Traverse into submenus or toggle choices.
* **Spacebar**: Toggle boolean options (``[*]`` for enabled, ``[ ]`` for disabled).
* **Tab / Left / Right**: Switch focus between selection list and action buttons
  (``<Select>``, ``<Save>`` and ``<Exit>``).
* **Escape / Exit**: Return to parent menu or prompt to save configuration upon
  exiting the root level.

When saving, the engine writes ``.config`` and invokes ``genheader.py`` to
synchronize ``include/generated/autoconf.h``.

Batch Configuration Engine (conf.py)
------------------------------------

For automated scripts, CI pipelines and headless builds, ``scripts/kconfig/conf.py``
provides command-line batch operations:

* **Default Configuration**:

  .. code-block:: bash

     make defconfig              # Uses arch/x86/configs/$(ARCH)_defconfig
     make ARCH=i686 defconfig    # Uses arch/x86/configs/i686_defconfig

* **Pure Ring 0 Configuration**:

  .. code-block:: bash

     make pure_defconfig         # Disables CONFIG_USERLAND for supervisor testing

* **Savedefconfig Extraction**:

  .. code-block:: bash

     make savedefconfig          # Writes minimal diff against defaults to defconfig

* **Validation (oldconfig)**:

  .. code-block:: bash

     make oldconfig              # Validates and updates existing .config

Autoconf C Header Generation (genheader.py)
-------------------------------------------

The utility ``scripts/kconfig/genheader.py`` reads ``.config`` and emits
``include/generated/autoconf.h``, protecting each symbol under preprocessor
guards:

.. code-block:: c

   #ifndef _KEIRA_AUTOCONF_H
   #define _KEIRA_AUTOCONF_H

   #define CONFIG_ARCH_X86_64 1
   #define CONFIG_ASLR 1
   #define CONFIG_CRYPTO_ENTROPY 1
   #define CONFIG_DISK_SIZE 32
   #define CONFIG_FS_EXT4 1
   #define CONFIG_FS_FAT 1
   #define CONFIG_FS_PROCFS 1
   #define CONFIG_IO_URING 1
   #define CONFIG_MAX_CPUS 2
   #define CONFIG_NET_E1000 1
   #define CONFIG_NET_SOCKETS 1
   #define CONFIG_NET_STACK 1
   #define CONFIG_QEMU_CPUS 2
   #define CONFIG_QEMU_MEM "128M"
   #define CONFIG_RLIMIT 1
   #define CONFIG_SCHED_WORK_STEALING 1
   #define CONFIG_SMP 1
   #define CONFIG_USERLAND 1
   #define CONFIG_USERLAND_COMPILER_KCC 1
   #define CONFIG_USERLAND_TESTS 1
   #define CONFIG_VERBOSE 0
   #define CONFIG_W_XOR_X 1

   #endif /* _KEIRA_AUTOCONF_H */

This header is included across the Ring 3 C runtime, system utilities and tests
via the compiler flag ``-I../include``.

Defconfig Profiles & Multi-Architecture Integration
---------------------------------------------------

The configuration system maintains strict dual-architecture parity across
``x86_64`` and ``i686``:

* **x86_64 Default** (``arch/x86/configs/x86_64_defconfig``): Configures 64-bit
  long mode, 2-core SMP, ASLR, W^X mitigations, userland utilities and compiler.
* **i686 Default** (``arch/x86/configs/i686_defconfig``): Configures 32-bit
  protected mode, PAE paging, 2-core SMP, full userland and diagnostic tools.
* **Pure Kernel Default** (``arch/x86/configs/pure_defconfig``): Configures a
  freestanding Ring 0 supervisor console image with all userspace staging
  disabled (``CONFIG_USERLAND=n``).

Helper Scripts & Code Quality Tools
-----------------------------------

Alongside the Kconfig parser, the ``scripts/`` directory houses essential
developer utilities modeled after Linux kernel standards:

* **Maintainer Lookup** (``scripts/get_maintainer.py``): Queries subsystem
  maintainership matching file paths and glob patterns from ``MAINTAINERS``.
* **Patch & Style Verification** (``scripts/checkpatch.py``): Enforces strict
  formatting rules, trailing whitespace detection, SPDX license identifier
  compliance and prose style standards.

Build System & Makefile Targets
-------------------------------

The root ``Makefile`` integrates Kconfig into the primary build lifecycle:

+-----------------------------+----------------------------------------------------+
| Makefile Target             | Operational Description                            |
+=============================+====================================================+
| ``make menuconfig``         | Launch interactive curses configuration editor     |
+-----------------------------+----------------------------------------------------+
| ``make defconfig``          | Generate baseline .config for current target ARCH  |
+-----------------------------+----------------------------------------------------+
| ``make pure_defconfig``     | Generate pure Ring 0 kernel configuration          |
+-----------------------------+----------------------------------------------------+
| ``make savedefconfig``      | Save minimal configuration diff to defconfig       |
+-----------------------------+----------------------------------------------------+
| ``make oldconfig``          | Validate and refresh existing .config              |
+-----------------------------+----------------------------------------------------+
| ``make mrproper``           | Pristine clean removing build artifacts and .config|
+-----------------------------+----------------------------------------------------+
