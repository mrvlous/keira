<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Fast Userspace Mutex (Futex)

This document details userspace synchronization primitives and kernel-managed wait queues in Keira Kernel.

---

## Operations Supported

| Opcode | Constant | Action |
| :--- | :--- | :--- |
| `0` | `FUTEX_WAIT` | Suspend task if `*uaddr == val` until woken |
| `1` | `FUTEX_WAKE` | Wake up to `val` tasks suspended on `uaddr` |
| `2` | `FUTEX_REQUEUE` | Wake tasks and requeue remaining tasks to another futex word |

---

## Core API (`crates/ipc/src/futex.rs`)

```rust
pub fn sys_futex(uaddr: *const u32, op: i32, val: u32, timeout: u64) -> Result<i32, &'static str>;
```
