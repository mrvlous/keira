<!--
SPDX-License-Identifier: GPL-2.0-only

Keira Kernel - Operating System Kernel
Copyright (C) 2026 Moh. Ananda Firmansyah Putra
-->

# POSIX Shared Memory IPC & Counting Semaphore Engine

This document details the shared memory IPC (`shmget`/`shmat`) and counting semaphore subsystem in Keira Kernel.

## 1. Subsystem Architecture
The Shared Memory IPC engine ([shm.rs](../../kernel/src/ipc/shm.rs)) provides zero-copy inter-process communication by mapping shared physical memory pages directly into the virtual address spaces of multiple process tasks.

*   **Shared Memory Table (`SHM_TABLE`)**: Manages allocated physical page frames and attach counts (`nattch`).
*   **Counting Semaphores**: Manages atomic integer counters and thread wait queues for mutual exclusion.

---

## 2. Shared Memory Operations
*   **`shmget`**: Allocates or locates a shared memory segment for a given 32-bit key.
*   **`shmat`**: Maps the physical pages of a shared memory segment into the caller's Ring 3 virtual memory space (`0x70000000..`).
*   **`shmdt`**: Unmaps the virtual pages and decrements the segment attach count.

---

## 3. System Call & Shell Commands
*   **System Call 75 (`sys_shm_sem`)**: `(cmd: u32, arg1: u64, arg2: u64) -> status`
*   **`ipcs`**: Query active shared memory segments and semaphores.
*   **`ipcrm`**: Remove shared memory segments or semaphore resources by ID.
