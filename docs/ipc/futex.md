<!--
SPDX-License-Identifier: GPL-2.0-only

Keira Kernel - Operating System Kernel
Copyright (C) 2026 Moh. Ananda Firmansyah Putra
-->

# Fast Userspace Mutex (Futex) Subsystem

This document details fast userspace synchronization, kernel wait queues, and POSIX thread synchronization in Keira Kernel.

---

## 1. Subsystem Overview

Keira Kernel implements Fast Userspace Mutexes (**Syscall 40 `sys_futex`**) enabling user-mode atomic locks to execute without kernel context switches unless contention occurs.

---

## 2. System Call Interface

```c
// Syscall 40: Futex wait/wake operation
long sys_futex(uint32_t *uaddr, int op, uint32_t val, uint64_t timeout_ptr);
```

---

## 3. Operations

*   `FUTEX_WAIT`: Puts the calling task to sleep if `*uaddr == val`.
*   `FUTEX_WAKE`: Wakes up to `val` tasks waiting on physical address `uaddr`.
