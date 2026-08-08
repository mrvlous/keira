<!--
SPDX-License-Identifier: GPL-2.0-only

Keira Kernel - Operating System Kernel
Copyright (C) 2026 Moh. Ananda Firmansyah Putra
-->

# Kernel Page Table Isolation (KPTI / KASI) Security Subsystem

This document details the Kernel Page Table Isolation (KPTI / KASI) security architecture in Keira Kernel.

## Overview
The KPTI engine, implemented in [kpti.rs](../../kernel/src/mem/kpti.rs), isolates kernel memory page tables separating Ring 0 and Ring 3 virtual address spaces via **Syscall 69 (`sys_kpti`)**.

## Architectural Features
*   **Ring 3 Memory Isolation**: Prevents speculative side-channel attacks (Meltdown / Spectre) by unmapping kernel pages in user mode.
*   **Shadow Page Tables**: Maintains dual PML4 page tables for kernel and user contexts.

---
*Back to [Architecture Index](../README.md)*
