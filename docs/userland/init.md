<!-- SPDX-License-Identifier: GPL-2.0-only -->

<!--
SPDX-License-Identifier: GPL-2.0-only

Keira Kernel - Operating System Kernel
Copyright (C) 2026 Moh. Ananda Firmansyah Putra
-->

# System Initialization & Userland Process Bootstrap

This document details the kernel initialization sequence and interactive shell bootstrapping during system boot.

## 1. Kernel Bootstrapping
After completing early hardware initialization, memory management, and storage mounting, the kernel prepares the execution environment:
*   **Kernel Entry**: [entry.rs](../../crates/kernel/src/entry.rs)
*   **Virtual Memory**: Page-Map Level-4 (PML4) paging structures are configured with isolated user-space segments and Ring 3 protection.
*   **Task State Segment (TSS)**: Configured with dedicated `RSP0` kernel privilege transition stacks and Model Specific Registers (STAR, LSTAR, SFMASK) for `syscall` / `sysret`.

---

## 2. Interactive Shell Bootstrap
The kernel shell subsystem performs the initial user session initialization:
1.  **Banner & Terminal Display**: Outputs kernel version and active TTY identifier (`tty1`).
2.  **Startup Script Execution**: Executes `/config/boot/boot.cfg` and environment setup.
3.  **Prompt Dispatch**: Initializes default user session (`admin@keira ~ >`) and enters the interrupt-driven event loop.

---

## 3. Userland Program Execution
User applications (such as [kcc](../../user/bin/kcc/main.c)) are launched directly from disk/initrd:
*   **Invocation**: Via the `run` command or `sys_exec` / `sys_spawn` system calls.
*   **Address Space**: Isolated address space cloned from kernel page tables with a dedicated 64 KB user stack.
*   **Privilege Level**: Executes in Ring 3 (User Mode) with fast syscall dispatch.
