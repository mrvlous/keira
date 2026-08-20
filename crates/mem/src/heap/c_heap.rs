// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Foreign Function Interface (FFI) bindings to the C kernel bump heap allocator (`mm/heap.c`).

extern "C" {
    /// Initialize the C kernel bump heap allocator with a start address and size.
    pub fn heap_init(start: *mut u8, size: usize);
    /// Allocate a contiguous memory block from the C kernel heap.
    pub fn kmalloc(size: usize) -> *mut u8;
    /// Free memory block (stub in sequential bump allocator).
    pub fn kfree(ptr: *mut u8);
    /// Get the total number of allocation requests.
    pub fn heap_get_alloc_count() -> usize;
    /// Get peak heap memory usage in bytes.
    pub fn heap_get_peak() -> usize;
}
