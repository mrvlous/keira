.. SPDX-License-Identifier: GPL-2.0-only


Keira Userland Architecture, Build System & Packaging
=====================================================

The ``userland`` domain (KeiraBox) provides a freestanding, modular Ring 3 userspace environment hosted directly within the Keira monolithic kernel repository. Inspired by the applet design of BusyBox, KeiraBox delivers an ultra-compact, dependency-free collection of 46 system utilities, a freestanding C standard runtime library (``libc.a``), a native C compiler toolchain (``kcc``) and comprehensive ABI test suites.

1. Architectural Principles & Kernel Demarcation
================================================

Keira enforces a strict separation between Ring 0 supervisor execution and Ring 3 userspace runtime:

- **Freestanding Kernel Independence (USERLAND=0)**: The kernel compiles and boots completely self-contained in pure Ring 0 supervisor mode. It features an in-kernel rescue shell and a minimal root filesystem without any userland binaries.
- **Integrated Userland Suite (USERLAND=1)**: When enabled, the build system compiles the full KeiraBox suite and installs utilities into ``/bin``, headers into ``/include``, libraries into ``/lib`` and system scripts into ``/etc``.
- **Zero Bloat Model**: Keira operates strictly with 32-bit numerical credentials in Ring 0 (UID 0 for superuser, UID 1000 for regular user contexts). The system omits traditional distribution bloat such as ``/etc/passwd``, ``/etc/shadow``, ``/etc/group`` and PAM modules.

.. code-block:: text

   graph TD
       Kernel["Keira Monolithic Kernel (crates/)"] <--> Syscall["Syscall Dispatcher (INT 0x80 / SYSCALL)"]
       Syscall <--> Libc["libc/<br/>Freestanding C Runtime & Headers"]
       Libc --> Core["coreutils/<br/>14 File & Text Utilities"]
       Libc --> Proc["procutils/<br/>3 Process & Memory Utilities"]
       Libc --> Sys["sysutils/<br/>11 System Control Utilities"]
       Libc --> Init["init/<br/>3 Supervisor & Shutdown Utilities"]
       Libc --> Shell["shell/<br/>POSIX Interactive Shell"]
       Libc --> Net["netutils/<br/>7 Network & HTTP Utilities"]
       Libc --> Toolchain["toolchain/<br/>KCC Native C Compiler"]
       Libc --> Tests["tests/<br/>6 ABI & Stress Test Suites"]
       Init --> Config["etc/<br/>Bootstrap Scripts & Auto Configs"]

2. Modular Domain Organization
==============================

The ``userland/`` tree is partitioned into specialized domains to maximize maintainability and modularity:

.. list-table::
   :header-rows: 1
   :widths: 18 14 68

   * - Domain Directory
     - Binary Count
     - Scope and Included Executables
   * - ``coreutils/``
     - 14 binaries
     - Fundamental POSIX file and text processing: ``cat``, ``chmod``, ``chown``, ``clear``, ``echo``, ``grep``, ``head``, ``ln``, ``ls``, ``mkdir``, ``rm``, ``tail``, ``touch`` and ``wc``.
   * - ``procutils/``
     - 3 binaries
     - Process table telemetry, memory mapping and signal dispatch: ``kill``, ``pmap`` and ``ps``.
   * - ``sysutils/``
     - 11 binaries
     - System administration, VFS mount operations and diagnostics: ``df``, ``dmesg``, ``hostname``, ``id``, ``mount``, ``sleep``, ``su``, ``sysinfo``, ``umount``, ``uname`` and ``whoami``.
   * - ``init/``
     - 3 binaries
     - System lifecycle management and PID 1 supervision: ``init``, ``poweroff`` and ``reboot``.
   * - ``shell/``
     - 1 binary
     - Standalone Ring 3 interactive command interpreter: ``sh``.
   * - ``netutils/``
     - 7 binaries
     - Socket telemetry, interface configuration and HTTP client: ``arp``, ``fetch``, ``ifconfig``, ``iptables``, ``netstat``, ``ping`` and ``route``.
   * - ``toolchain/``
     - 1 binary
     - Native C compiler suite: ``kcc`` (preprocessor, recursive descent parser, AST, codegen and ELF emitter).
   * - ``tests/``
     - 6 test suites
     - ABI conformance and fuzzing harnesses: ``test_abi``, ``fuzz_abi``, ``test_pipe``, ``test_poll``, ``test_select`` and ``test_threads``.
   * - ``libc/``
     - Static Library
     - Freestanding ISO C and POSIX runtime routines, CRT0 startup stubs (``arch/``) and 44 standard headers (``include/``).
   * - ``etc/``
     - Source Scripts
     - Canonical bootstrap scripts: ``init.sh``, ``profile`` and ``inittab``.

3. Dynamic Configuration Auto-Generation
========================================

System configuration files containing build-specific or architecture-specific attributes are generated dynamically by the master ``Makefile`` during root filesystem staging. This eliminates static, unmaintained text files from version control:

- **``/etc/kernel.cfg``**: Automatically generated with active kernel name, target version, architecture and userland enablement mode.
- **``/etc/hostname``**: Initialized dynamically to ``keira\n``.
- **``/etc/os-release``**: Conforms to the standard freedesktop specification with build metadata.
- **``/etc/issue``**: Formatted console banner identifying kernel release and CPU architecture.
- **``/etc/hosts``**: Baseline loopback and local node resolution table.

Runtime state mutation (such as modifying the hostname via ``hostname <new_name>`` or mounting scratch volumes) is orchestrated dynamically by ``/etc/init.sh`` upon PID 1 bootstrap.

4. Build System Orchestration
=============================

The ``userland/Makefile`` provides targeted build recipes for cross-compilation, linting and verification:

.. code-block:: bash

   # Compile C library and all 46 userland binaries
   make -C userland ARCH=x86_64 bins

   # Format all source files according to clang-format rules
   make -C userland format

   # Perform static analysis on userland C code with clang-tidy
   make -C userland lint

   # Display binary footprint and static library archive sizes
   make -C userland size

   # Verify diagnostic and ABI test binaries
   make -C userland test

5. Contributor Workflow & Coding Standards
==========================================

When introducing new utilities or extending existing packages in ``userland/``, contributors must observe the following invariants:

1. **Freestanding C Compliance**: All userland code must compile with ``-ffreestanding -nostdlib -fno-stack-protector``. Never include host platform headers; use standard headers provided by ``userland/libc/include/``.
2. **License Specification**: Every new file must start with the canonical Linux kernel SPDX tag: ``// SPDX-License-Identifier: GPL-2.0-only`` for C sources or ``# SPDX-License-Identifier: GPL-2.0-only`` for scripts.
3. **Typography & Style Guidelines**: Strictly enforce zero Oxford commas and zero double whitespaces across all source comments and documentation text.
4. **Domain Placement**:
   - Single-file utilities: Place the source file directly inside the matching domain (``coreutils/``, ``procutils/``, ``sysutils/``, ``init/``, ``shell/`` or ``netutils/``) and append the executable name to the corresponding list in ``userland/Makefile``.
   - Multi-file utilities: Structure complex applications with internal ``include/`` and modular subdirectories (following the pattern of ``sysutils/sysinfo/`` and ``toolchain/kcc/``), then register via the ``MULTI_BIN_RULE`` macro.
