<!--
SPDX-License-Identifier: GPL-2.0-only

Keira Kernel - Operating System Kernel
Copyright (C) 2026 Moh. Ananda Firmansyah Putra
-->

# KASAN (Kernel Address Sanitizer) Shadow Memory Diagnostic Engine

This document details shadow memory validation, out-of-bounds access detection, and use-after-free diagnostics in Keira Kernel.

---

## 1. Subsystem Overview

Keira Kernel provides Kernel Address Sanitizer ([kasan.rs](../../kernel/src/mem/kasan.rs)) utilizing shadow memory banks at `0xD0000000` to validate memory accesses.

---

## 2. System Call Interface

```c
// Syscall 57: Validate memory address against KASAN shadow memory
long sys_kasan(uint64_t addr, size_t size);
```

---

## 3. Kernel APIs

*   `pub fn sys_kasan(addr: u64, size: usize) -> Result<u64, &'static str>`: Verifies heap memory access safety.
