<!--
SPDX-License-Identifier: GPL-2.0-only

Keira Kernel - Operating System Kernel
Copyright (C) 2026 Moh. Ananda Firmansyah Putra
-->

# POSIX Message Queue IPC Subsystem

This document details priority-based in-kernel message queues for inter-process communication in Keira Kernel.

---

## 1. Subsystem Overview

Keira Kernel implements POSIX Message Queues ([mqueue.rs](../../kernel/src/ipc/mqueue.rs)) providing asynchronous message passing between processes.

---

## 2. System Call Interface

```c
// Syscall 58: Open or create a POSIX message queue
long sys_mq_open(const char *name_ptr, int oflag, uint32_t mode);
```

---

## 3. Kernel APIs

*   `pub fn sys_mq_open(name_ptr: *const u8, oflag: i32, mode: u32) -> Result<u64, &'static str>`: Allocates a new priority message queue.
