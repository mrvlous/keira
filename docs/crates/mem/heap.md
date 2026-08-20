<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Kernel Dynamic Heap Allocator

Documentation for the kernel heap allocator in [`crates/mem/src/heap/`](../../../crates/mem/src/heap).

## Architecture
- Initialized in `1MB` kernel memory space during early Rust kernel initialization.
- Provides `kmalloc(size: usize) -> *mut u8` and `kfree(ptr: *mut u8)`.
- Tracks total allocations, peak usage, and active allocation counts to prevent kernel memory leaks.
