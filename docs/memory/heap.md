<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Kernel Heap Allocator (`kmalloc` / `kfree`)

This document details the 16-byte aligned kernel heap allocator used for dynamic runtime memory allocations in Keira Kernel.

---

## Heap Specifications

* **Heap Base**: `0xFFFF800000000000` (64-bit) / `0xC1000000` (32-bit).
* **Alignment**: 16-byte aligned on all allocations.
* **Concurrency**: Thread-safe with atomic pointer tracking (`AtomicPtr`) and compare-and-swap (CAS).

---

## Core API (`crates/mem/src/heap/mod.rs`)

```rust
/// Allocate a block of kernel memory.
#[no_mangle]
pub extern "C" fn kmalloc(size: usize) -> *mut u8;

/// Free a previously allocated memory block.
#[no_mangle]
pub extern "C" fn kfree(ptr: *mut u8);

/// Query heap memory statistics.
pub fn heap_get_used() -> usize;
pub fn heap_get_free() -> usize;
pub fn heap_get_total() -> usize;
pub fn heap_get_peak() -> usize;
pub fn heap_get_alloc_count() -> usize;
```
