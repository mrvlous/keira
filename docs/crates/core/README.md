<!-- SPDX-License-Identifier: GPL-2.0-only -->

# `keira-core` - Foundational Kernel Primitives

The `keira-core` crate provides zero-dependency, freestanding (`#![no_std]`) data structures, synchronization primitives, memory alignment routines, error taxonomy, and circular diagnostic logging.

## Modules

| Module | Source Link | Purpose |
| :--- | :--- | :--- |
| **`collections`** | [`collections/`](../../../crates/core/src/collections) | Fixed-capacity ring buffers and LRU sector caches |
| **`sync`** | [`sync/`](../../../crates/core/src/sync) | Re-entrant `SpinLock` and atomic `SpinMutex` guards |
| **`mem`** | [`mem/`](../../../crates/core/src/mem) | Hardware page alignment and bit manipulation helpers |
| **`log`** | [`log/`](../../../crates/core/src/log) | In-memory circular `klog` dmesg diagnostic buffer |
| **`error`** | [`error.rs`](../../../crates/core/src/error.rs) | Unified `KernelError` enumeration and `KernelResult<T>` |

## Design Invariants

- **Zero Allocation**: Every structure in `keira-core` operates exclusively in statically bounded memory without requiring an active heap allocator.
- **Freestanding Safety**: All synchronization primitives avoid OS scheduling dependencies, relying solely on CPU atomic instructions and interrupt state preservation.
