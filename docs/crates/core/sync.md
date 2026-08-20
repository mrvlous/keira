<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Core Synchronization Primitives

Documentation for thread and interrupt synchronization mechanisms in [`crates/core/src/sync/`](../../../crates/core/src/sync).

## 1. `SpinLock`

Ticket-style spinlock utilizing `core::sync::atomic::AtomicBool` for mutual exclusion across CPU cores.

```rust
use keira_core::sync::SpinLock;

static DRIVER_LOCK: SpinLock = SpinLock::new();

fn access_hardware() {
    let _guard = DRIVER_LOCK.lock();
    // Critical section - exclusive access guaranteed
}
```

## 2. `SpinMutex<T>` & `SpinMutexGuard<T>`

Safe container wrapping shared mutable data inside a `SpinLock`. Implements `Deref` and `DerefMut` for RAII guard unlocking on drop.

### Safety Invariants
- `SpinMutex` must not be acquired recursively by the same execution thread to avoid deadlocks.
- Interrupt-context routines must disable interrupts (`cli`) before acquiring mutexes shared with task contexts.
