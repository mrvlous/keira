// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Hardened kernel heap allocator with segregated size classes and canaries.
//!
//! Subdivided into specialized modules for memory pool allocation and telemetry metrics.

pub mod pool;

#[cfg(test)]
pub use pool::HEAP_TEST_MUTEX;
pub use pool::{
    class_index_for_size, heap_get_active_alloc_count, heap_get_alloc_count, heap_get_arena_used,
    heap_get_free, heap_get_peak, heap_get_telemetry, heap_get_total, heap_get_used, heap_init,
    heap_stress_test, kfree, kmalloc, BlockHeader, BLOCK_MAGIC, HEADER_SIZE, HEAP_ALIGNMENT,
    HEAP_ALIGN_MASK, LARGE_CLASS, NUM_SIZE_CLASSES, SIZE_CLASSES,
};

#[cfg(test)]
mod tests;
