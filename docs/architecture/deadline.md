<!--
SPDX-License-Identifier: GPL-2.0-only

Keira Kernel - Operating System Kernel
Copyright (C) 2026 Moh. Ananda Firmansyah Putra
-->

# POSIX Sched_Deadline EDF Hard Real-Time Scheduler Policy

This document details Earliest Deadline First (EDF) hard real-time task scheduling in Keira Kernel.

---

## 1. Subsystem Overview

Keira Kernel provides an EDF real-time task scheduler policy ([deadline.rs](../../kernel/src/task/deadline.rs), **Syscall 64 `sys_sched_setattr`**) guaranteeing nanosecond execution deadlines for critical kernel tasks.

---

## 2. System Call Interface

```c
// Syscall 64: Set task real-time scheduling attributes
long sys_sched_setattr(uint32_t pid, uint64_t attr_ptr, uint32_t flags);
```

---

## 3. Kernel APIs

*   `pub fn sys_sched_setattr(pid: u32, attr_ptr: u64, flags: u32) -> Result<u64, &'static str>`: Configures runtime, deadline, and period parameters for hard real-time tasks.
