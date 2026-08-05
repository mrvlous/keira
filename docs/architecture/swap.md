# Swap Space & Virtual Memory Pager Subsystem

This document details anonymous physical memory page swapping to disk partitions and swap management system calls in Keira Kernel.

---

## 1. Subsystem Overview

Keira Kernel implements Virtual Memory Swapping ([swap.rs](../../kernel/src/mem/swap.rs)) offloading unreferenced physical RAM pages to dedicated disk partitions when memory capacity limits are reached.

---

## 2. System Call Interface

```c
// Syscall 53: Activate swap partition on storage device
long sys_swapon(const char *path_ptr, int swapflags);

// Syscall 54: Deactivate swap partition
long sys_swapoff(const char *path_ptr);
```

---

## 3. Kernel APIs

*   `pub fn sys_swapon(path_ptr: *const u8, swapflags: i32) -> Result<u64, &'static str>`: Mounts and formats a 256MB swap partition.
*   `pub fn sys_swapoff(path_ptr: *const u8) -> Result<u64, &'static str>`: Unmounts active swap partitions.
