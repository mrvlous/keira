// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Kernel heap memory accounting, capacity metrics, and telemetry queries.

use super::allocator::{HEAP_END, HEAP_NEXT, HEAP_START};
use core::sync::atomic::{AtomicUsize, Ordering};

pub(crate) static ALLOC_COUNT: AtomicUsize = AtomicUsize::new(0);
pub(crate) static ACTIVE_ALLOC_COUNT: AtomicUsize = AtomicUsize::new(0);
pub(crate) static CURRENT_USED: AtomicUsize = AtomicUsize::new(0);
pub(crate) static PEAK_USED: AtomicUsize = AtomicUsize::new(0);

/// Retrieves total configured capacity of kernel heap in bytes.
#[no_mangle]
pub extern "C" fn heap_get_total() -> usize {
    let start = HEAP_START.load(Ordering::SeqCst) as usize;
    let end = HEAP_END.load(Ordering::SeqCst) as usize;
    if end > start {
        end - start
    } else {
        0
    }
}

/// Retrieves currently allocated active bytes in kernel heap.
#[no_mangle]
pub extern "C" fn heap_get_used() -> usize {
    CURRENT_USED.load(Ordering::SeqCst)
}

/// Retrieves remaining unallocated free bytes in kernel heap.
#[no_mangle]
pub extern "C" fn heap_get_free() -> usize {
    let total = heap_get_total();
    let used = heap_get_used();
    total.saturating_sub(used)
}

/// Retrieves total number of allocation requests since initialization.
#[no_mangle]
pub extern "C" fn heap_get_alloc_count() -> usize {
    ALLOC_COUNT.load(Ordering::SeqCst)
}

/// Retrieves number of currently active (unfreed) memory blocks.
#[no_mangle]
pub extern "C" fn heap_get_active_alloc_count() -> usize {
    ACTIVE_ALLOC_COUNT.load(Ordering::SeqCst)
}

/// Retrieves peak heap memory usage in bytes.
#[no_mangle]
pub extern "C" fn heap_get_peak() -> usize {
    PEAK_USED.load(Ordering::SeqCst)
}

/// Retrieves total bytes consumed from underlying bump arena pool.
#[no_mangle]
pub extern "C" fn heap_get_arena_used() -> usize {
    let start = HEAP_START.load(Ordering::SeqCst) as usize;
    let next = HEAP_NEXT.load(Ordering::SeqCst) as usize;
    if next > start {
        next - start
    } else {
        0
    }
}
