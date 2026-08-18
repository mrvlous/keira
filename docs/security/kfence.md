<!--
SPDX-License-Identifier: GPL-2.0-only

Keira Kernel - Operating System Kernel
Copyright (C) 2026 Moh. Ananda Firmansyah Putra
-->

# KFENCE (Kernel Electric Fence) Sampling Memory Guard Engine

This document details low-overhead sampling memory guards and out-of-bounds guard pages in Keira Kernel.

---

## 1. Subsystem Overview

Keira Kernel implements KFENCE ([kfence.rs](../../kernel/src/mem/kfence.rs), **Syscall 63 `sys_kfence`**) utilizing out-of-bounds guard pages to catch memory corruption, use-after-free, and double-free bugs in production heap allocations.

---

## 2. System Call Interface

```c
// Syscall 63: Query or configure KFENCE sampling memory guard status
long sys_kfence(unsigned int sample_interval, unsigned int flags);
```

---

## 3. Kernel APIs

*   `pub fn sys_kfence(sample_interval: u32, flags: u32) -> Result<u64, &'static str>`: Configures heap guard sampling intervals and reports corruption events.
