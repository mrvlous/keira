<!--
SPDX-License-Identifier: GPL-2.0-only

Keira Kernel - Operating System Kernel
Copyright (C) 2026 Moh. Ananda Firmansyah Putra
-->

# Seccomp BPF System Call Filter Engine

This document details in-kernel BPF system call sandbox filtering, process privilege bounding, and seccomp system call execution in Keira Kernel.

---

## 1. Subsystem Overview

Keira Kernel provides Secure Computing ([seccomp.rs](../../kernel/src/task/seccomp.rs)) evaluating BPF rules on every system call entry.

---

## 2. System Call Interface

```c
// Syscall 52: Enforce seccomp BPF system call filter
long sys_seccomp(uint32_t op, uint32_t flags, uint64_t args_ptr);
```

---

## 3. Kernel APIs

*   `pub fn sys_seccomp(op: u32, flags: u32, args_ptr: u64) -> Result<u64, &'static str>`: Activates system call filtering sandbox.
