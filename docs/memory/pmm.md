<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Physical Memory Manager (PMM)

This document specifies the bitmap-based physical frame allocator responsible for managing all available physical RAM in Keira Kernel.

---

## Technical Specifications

* **Frame Size**: 4096 bytes (`4 KB`).
* **Bitmap Granularity**: 1 bit per 4096-byte frame (0 = Available, 1 = Allocated).
* **Frame Alignment**: 4096-byte page boundary.
* **Bitmap Storage**: Statically reserved in the kernel BSS segment.

---

## Core API (`crates/mem/src/pmm/mod.rs`)

```rust
/// Allocate a single 4KB physical frame.
pub fn alloc_frame() -> Option<usize>;

/// Free a previously allocated physical frame.
pub fn free_frame(frame: usize);

/// Allocate multiple contiguous physical frames (for DMA).
pub fn alloc_contiguous_frames(count: usize) -> Option<usize>;

/// Free contiguous physical frames.
pub fn free_contiguous_frames(start: usize, count: usize);

/// Query memory stats (total, free, allocated).
pub fn get_stats() -> (usize, usize, usize);
```
