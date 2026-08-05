# io_uring Async I/O Worker Thread Pool Engine

This document details kernel-space async I/O worker polling thread pools in Keira Kernel.

---

## 1. Subsystem Overview

Keira Kernel provides an async I/O worker thread pool ([io_worker.rs](../../kernel/src/ipc/io_worker.rs), **Syscall 62 `sys_io_uring_register`**) executing offloaded blocking storage and socket requests asynchronously.

---

## 2. System Call Interface

```c
// Syscall 62: Register buffers, files, or worker threads in an io_uring instance
long sys_io_uring_register(int fd, unsigned int opcode, uint64_t arg_ptr, unsigned int nr_args);
```

---

## 3. Kernel APIs

*   `pub fn sys_io_uring_register(fd: i32, opcode: u32, arg_ptr: u64, nr_args: u32) -> Result<u64, &'static str>`: Spawns and manages kernel polling worker threads.
