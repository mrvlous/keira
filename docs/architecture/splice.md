# Zero-Copy Kernel Pipe Splice Subsystem

This document details zero-copy page frame swapping between file descriptors and splice system calls in Keira Kernel.

---

## 1. Subsystem Overview

Keira Kernel provides zero-copy pipe splicing ([splice.rs](../../kernel/src/ipc/splice.rs)) moving data between file descriptors without userland memory copying.

---

## 2. System Call Interface

```c
// Syscall 47: Splice data between two file descriptors
long sys_splice(uint64_t fd_in, uint64_t fd_out, size_t len, uint32_t flags);

// Syscall 48: Splice memory vector pages into kernel pipe buffer
long sys_vmsplice(uint64_t fd, uint64_t iov_ptr, size_t nr_segs, uint32_t flags);
```

---

## 3. Kernel APIs

*   `pub fn sys_splice(fd_in: u64, fd_out: u64, len: usize, flags: u32) -> Result<usize, &'static str>`: Swaps physical page frames between pipes/file descriptors.
*   `pub fn sys_vmsplice(fd: u64, iov_ptr: u64, nr_segs: usize, flags: u32) -> Result<usize, &'static str>`: Maps userland memory vectors directly to kernel pipe buffers.
