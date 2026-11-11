.. SPDX-License-Identifier: GPL-2.0-only

.. list-table:: Keira Kernel Subsystem Language Standards
   :widths: 25 35 40
   :header-rows: 1

   * - Subsystem Layer
     - Language & Standard
     - Invariants & Tooling
   * - Ring 0 Kernel Core
     - 100% Pure Rust (``no_std``)
     - Zero panics, strict bounds checking, ``cargo clippy``
   * - Bootstrap & Low-Level Traps
     - x86/x86_64 Assembly (NASM)
     - Semicolon comments, 16-byte stack alignment
   * - Autonomous Userland (Ring 3)
     - C99 Freestanding
     - Linux kernel C style, 4-space indent, ``clang-tidy``
