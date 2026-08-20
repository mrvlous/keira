<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Fast Userspace Mutex (Futex)

Documentation for user space mutex synchronization in [`crates/ipc/src/futex/`](../../../crates/ipc/src/futex).

## System Call (`sys_futex` - Syscall 40)
- `FUTEX_WAIT`: Suspends calling thread if memory location equals expected value.
- `FUTEX_WAKE`: Wakes $N$ threads waiting on target futex address.
- `FUTEX_LOCK_PI` / `FUTEX_UNLOCK_PI`: Priority inheritance locking to prevent priority inversion.
