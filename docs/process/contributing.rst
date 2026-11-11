.. SPDX-License-Identifier: GPL-2.0-only

=================================
How to Contribute to Keira Kernel
=================================

Thank you for your interest in contributing to the Keira Kernel project! This document outlines the fundamental technical principles, development environment setup, architectural invariants and verification procedures required when proposing code or documentation changes.

Guiding Principles & Philosophy
===============================

Keira is a pure freestanding monolithic kernel implemented in safe Rust, Assembly and C. Development is guided by several foundational engineering rules:

1. **Pure Monolithic Kernel Demarcation**: Keira is an autonomous Ring 0 kernel. Userland runtime components (such as libc and core utilities) reside in ``userland/`` but maintain strict isolation from Ring 0 kernel crates.
2. **Dual-Architecture Parity**: Every change must compile cleanly and pass verification on both ``x86_64`` (64-bit Long Mode) and ``i686`` (32-bit Protected Mode) architectures.
3. **Freestanding Safety & Robustness**: Unhandled hardware exceptions or unexpected faults in Ring 3 userland processes must never crash or panic the kernel. Panic in Ring 0 is reserved strictly for unrecoverable hardware state faults.
4. **Zero Warnings & Errors Policy**: The entire codebase, including all 12 modular kernel crates, assembly entry stubs, userland C library and Sphinx documentation, must compile and lint with zero errors and zero warnings.
5. **SPDX License Compliance**: Every source and documentation file must contain a machine-readable SPDX license identifier header (``GPL-2.0-only`` for kernel sources and documentation).

Development Environment Setup
=============================

Ensure that host dependencies and cross-compilation toolchains are installed prior to developing:

* **Rust Toolchain**: Nightly Rust with ``rust-src``, ``llvm-tools-preview`` and ``rustfmt`` components installed.
* **C Toolchains**: GCC supporting both 64-bit (``-m64``) and 32-bit (``-m32``) code generation, Clang and LLVM utilities (``clang-format``, ``clang-tidy``).
* **Assembler**: NASM (Netwide Assembler) version 2.15 or newer.
* **Emulation & Testing**: QEMU system emulators (``qemu-system-x86_64`` and ``qemu-system-i386``).
* **Disk & ISO Packaging**: ``xorriso``, ``grub-mkrescue``, ``mtools`` and ``parted``.
* **Documentation**: Python 3 with Sphinx and dependencies listed in ``docs/sphinx/requirements.txt``.

Quality Gate Checklist
======================

Before submitting any patch or opening a pull request, run the comprehensive verification pipeline locally:

.. code-block:: bash

   # 1. Format Rust workspace and userland C source files
   make format

   # 2. Run static analysis on userland C code and documentation
   make lint

   # 3. Verify all documentation files against style guide and SPDX rules
   make docs-lint

   # 4. Execute all host unit tests across all 12 kernel crates
   make test-unit

   # 5. Build and verify userland diagnostic test binaries for both architectures
   make user-test ARCH=x86_64
   make user-test ARCH=i686

   # 6. Build kernel binaries and bootable ISO images for both architectures
   make full

   # 7. Execute automated headless smoke boot tests across all architectures
   make test-all

Submitting Your Contributions
=============================

* Review :doc:`submitting-patches` for detailed commit message formatting, git branch etiquette and Signed-off-by procedures.
* Consult :doc:`coding-style` for specific naming, indentation, comment rules and language guidelines.
* Check :doc:`build-system` for detailed make targets and configuration parameters.
* Review :doc:`maintainers` to identify appropriate subsystem maintainers and review scopes.
