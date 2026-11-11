.. SPDX-License-Identifier: GPL-2.0-only

=============================================
Guiding Principles & Architectural Invariants
=============================================

This document specifies the core architectural principles, hardware safety invariants and toolchain requirements governing all development on Keira Kernel.

Foundational Architecture Invariants
====================================

1. **Pure Monolithic Kernel Demarcation**: Keira is an autonomous Ring 0 monolithic kernel. Userland runtime components (such as libc and core utilities) reside in ``userland/`` and execute strictly in Ring 3 user mode.
2. **Dual-Architecture Parity**: Every change must compile cleanly and pass verification on both ``x86_64`` (64-bit Long Mode) and ``i686`` (32-bit Protected Mode) architectures with zero errors and zero warnings.
3. **Freestanding Safety & Fault Containment**: Unhandled hardware exceptions or unexpected faults in Ring 3 userland processes must never crash or panic the kernel. Panic in Ring 0 is reserved strictly for unrecoverable hardware state faults.
4. **Zero Warnings Policy**: All 12 modular kernel crates, assembly entry stubs, userland C library and Sphinx documentation must compile and lint with zero errors and zero warnings.
5. **SPDX License Compliance**: Every source and documentation file must contain a machine-readable SPDX license identifier header (``GPL-2.0-only`` for kernel sources and documentation).

Coding Standards & Hygiene
==========================

All contributions must adhere strictly to the following styling and structural invariants:

* **Zero Oxford Commas**: Omit commas before "and" or "or" in all prose, commit messages, documentation files and source code comments.
* **Zero Double Whitespaces**: Consecutive spaces between words are prohibited in all documentation, configuration files and comments.
* **No Decorative Comment Dividers**: Decorative comment dividers (such as repeated ``====`` or ``----``) are strictly forbidden in Rust, C and assembly sources. Heading underline decorations are reserved exclusively for reStructuredText (``.rst``) files.
* **Deterministic Formatting**: All Rust code must be formatted using ``cargo fmt``, all userland C code formatted using ``clang-format`` and all documentation verified using ``make docs-lint``.

Toolchain Prerequisites
=======================

Ensure that host dependencies and cross-compilation toolchains are installed prior to developing:

* **Rust Toolchain**: Nightly Rust with ``rust-src``, ``llvm-tools-preview`` and ``rustfmt`` components installed.
* **C Toolchains**: GCC supporting both 64-bit (``-m64``) and 32-bit (``-m32``) code generation, Clang and LLVM utilities (``clang-format``, ``clang-tidy``).
* **Assembler**: NASM (Netwide Assembler) version 2.15 or newer.
* **Emulation & Testing**: QEMU system emulators (``qemu-system-x86_64`` and ``qemu-system-i386``).
* **Disk & ISO Packaging**: ``xorriso``, ``grub-mkrescue``, ``mtools`` and ``parted``.
* **Documentation**: Python 3 with Sphinx and dependencies listed in ``docs/sphinx/requirements.txt``.
