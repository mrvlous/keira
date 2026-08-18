<!--
SPDX-License-Identifier: GPL-2.0-only

Keira Kernel - Operating System Kernel
Copyright (C) 2026 Moh. Ananda Firmansyah Putra
-->

# NX Bit & KASLR Hardware Security Subsystem

This document details hardware No-Execute (NX/XD) page protection and Kernel Address Space Layout Randomization (KASLR) in Keira Kernel.

---

## 1. Subsystem Overview

Keira Kernel enforces hardware No-Execute (NX) bit protection in 64-bit page tables to prevent userland buffer overflow code execution in stack or heap pages.

---

## 2. Page Table Flags

*   `PAGE_NX_BIT` (`1u64 << 63`): Sets No-Execute attribute bit on page table entries (PTE).
*   `KASLR_OFFSET`: Adds dynamic virtual address offset during early kernel initialization.

---

## 3. Kernel APIs

*   `pub fn enforce_nx_protection(vaddr: u64, size: usize)`: Applies NX flag to specified page ranges.
