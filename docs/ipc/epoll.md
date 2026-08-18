<!--
SPDX-License-Identifier: GPL-2.0-only

Keira Kernel - Operating System Kernel
Copyright (C) 2026 Moh. Ananda Firmansyah Putra
-->

# Epoll Scalable I/O Event Notification Engine

This document details $O(1)$ scalable event multiplexing, interest list management, and epoll system calls in Keira Kernel.

---

## 1. Subsystem Overview

Keira Kernel implements scalable I/O event polling ([epoll.rs](../../kernel/src/ipc/epoll.rs)) for monitoring large numbers of file descriptors.

---

## 2. System Call Interface

```c
// Syscall 55: Create an epoll event descriptor
long sys_epoll_create(int size);

// Syscall 56: Control file descriptors in epoll interest list
long sys_epoll_ctl(int epfd, int op, int fd, uint64_t event_ptr);
```

---

## 3. Kernel APIs

*   `pub fn sys_epoll_create(size: i32) -> Result<u64, &'static str>`: Allocates a new epoll multiplexer descriptor.
*   `pub fn sys_epoll_ctl(epfd: i32, op: i32, fd: i32, event_ptr: u64) -> Result<u64, &'static str>`: Adds/modifies/removes target descriptors.
