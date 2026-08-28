<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Fast Userspace Mutex (`futex`) Subsystem

This document specifies the Fast Userspace Mutex (futex) engine, hash-bucket wait queues, and atomic lock contention resolution in Keira Kernel.

---

## Futex Synchronization Flow

```mermaid
sequenceDiagram
    participant ThreadA as Userspace Thread A
    participant Word as Atomic Futex Word in Userspace RAM
    participant Kernel as Futex Subsystem (sys_futex)
    participant ThreadB as Userspace Thread B

    ThreadA->>Word: 1. Atomic Compare-And-Swap (Uncontended: 0 -> 1)
    Note over ThreadA: Acquired in Userland (Zero Syscalls)
    ThreadB->>Word: 2. Contention Detected (Word already 1)
    ThreadB->>Kernel: 3. sys_futex(uaddr, FUTEX_WAIT, 1)
    Note over Kernel: Put Thread B to Sleep on Futex Hash Bucket
    ThreadA->>Kernel: 4. sys_futex(uaddr, FUTEX_WAKE, 1)
    Kernel-->>ThreadB: 5. Wake up Thread B to retry acquisition
```

---

## Technical Specifications

| Parameter | Specification | Description |
| :--- | :--- | :--- |
| **Hash Buckets** | 64 Futex Wait Queues | Hash-indexed wait queues by physical page address |
| **Operations** | `FUTEX_WAIT`, `FUTEX_WAKE`, `FUTEX_REQUEUE` | Standard Linux futex operations |
| **Lock Latency** | Sub-microsecond uncontended | Zero kernel transitions when no contention |

---

## Core API (`crates/ipc/src/futex/mod.rs`)

```rust
pub const FUTEX_WAIT: u32 = 0;
pub const FUTEX_WAKE: u32 = 1;
pub const FUTEX_REQUEUE: u32 = 3;

/// Handle futex system call for userspace locking primitives.
pub unsafe fn sys_futex(
    uaddr: usize,
    futex_op: u32,
    val: u32,
    timeout_ms: u64,
) -> Result<i32, &'static str>;
```
