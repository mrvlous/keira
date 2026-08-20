<!-- SPDX-License-Identifier: GPL-2.0-only -->

# POSIX Shared Memory & Semaphores

Documentation for shared memory in [`crates/ipc/src/shm/`](../../../crates/ipc/src/shm).

## System Call (`sys_shm_sem` - Syscall 38)
- Maps shared physical page frames into the virtual address spaces of multiple processes for zero-copy IPC.
- Implements atomic semaphore locking commands (`SHM_CMD_GET`, `SHM_CMD_AT`, `SHM_CMD_DT`, `SHM_CMD_RM`).
