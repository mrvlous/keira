<!--
SPDX-License-Identifier: GPL-2.0-only

Keira Kernel - Operating System Kernel
Copyright (C) 2026 Moh. Ananda Firmansyah Putra
-->

# EventFD & SignalFD Notification Subsystem

This document details counter notification file descriptors, asynchronous event signaling, and POSIX signal routing via file descriptors in Keira Kernel.

---

## 1. Subsystem Overview

Keira Kernel provides kernel notification descriptors ([eventfd.rs](../../kernel/src/ipc/eventfd.rs)) for inter-thread event notification and signal handling.

---

## 2. System Call Interface

```c
// Syscall 50: Create an eventfd notification descriptor
long sys_eventfd(uint32_t init_val, uint32_t flags);

// Syscall 51: Create a signalfd descriptor for POSIX signals
long sys_signalfd(int fd, uint64_t mask, uint32_t flags);
```

---

## 3. Kernel APIs

*   `pub fn sys_eventfd(init_val: u32, flags: u32) -> Result<u64, &'static str>`: Allocates a 64-bit event counter descriptor.
*   `pub fn sys_signalfd(fd: i32, mask: u64, flags: u32) -> Result<u64, &'static str>`: Binds POSIX signal masks to a file descriptor.
