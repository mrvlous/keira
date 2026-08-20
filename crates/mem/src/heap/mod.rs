// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Pure Rust sequential bump heap allocator for early kernel boot memory requests.

use core::sync::atomic::{AtomicPtr, AtomicUsize, Ordering};

const HEAP_ALIGNMENT: usize = 16;
const HEAP_ALIGN_MASK: usize = HEAP_ALIGNMENT - 1;

static HEAP_START: AtomicPtr<u8> = AtomicPtr::new(core::ptr::null_mut());
static HEAP_END: AtomicPtr<u8> = AtomicPtr::new(core::ptr::null_mut());
static HEAP_NEXT: AtomicPtr<u8> = AtomicPtr::new(core::ptr::null_mut());
static ALLOC_COUNT: AtomicUsize = AtomicUsize::new(0);
static PEAK_USED: AtomicUsize = AtomicUsize::new(0);

/// Initialize the pure Rust kernel bump heap allocator with a start address and size.
#[no_mangle]
pub extern "C" fn heap_init(start: *mut u8, size: usize) {
    HEAP_START.store(start, Ordering::SeqCst);
    let end = unsafe { start.add(size) };
    HEAP_END.store(end, Ordering::SeqCst);
    HEAP_NEXT.store(start, Ordering::SeqCst);
    ALLOC_COUNT.store(0, Ordering::SeqCst);
    PEAK_USED.store(0, Ordering::SeqCst);
}

/// Allocate a contiguous memory block from the kernel heap with 16-byte alignment.
#[no_mangle]
pub extern "C" fn kmalloc(size: usize) -> *mut u8 {
    if size == 0 || size > (usize::MAX - HEAP_ALIGN_MASK) {
        return core::ptr::null_mut();
    }

    let aligned_size = (size + HEAP_ALIGN_MASK) & !HEAP_ALIGN_MASK;

    let start = HEAP_START.load(Ordering::SeqCst);
    let end = HEAP_END.load(Ordering::SeqCst);
    let current = HEAP_NEXT.load(Ordering::SeqCst);

    if current.is_null() || end.is_null() {
        return core::ptr::null_mut();
    }

    let current_addr = current as usize;
    let end_addr = end as usize;

    if current_addr + aligned_size < current_addr || current_addr + aligned_size > end_addr {
        return core::ptr::null_mut();
    }

    let new_next = (current_addr + aligned_size) as *mut u8;
    HEAP_NEXT.store(new_next, Ordering::SeqCst);
    ALLOC_COUNT.fetch_add(1, Ordering::SeqCst);

    let used = (new_next as usize) - (start as usize);
    let mut peak = PEAK_USED.load(Ordering::Relaxed);
    while used > peak {
        match PEAK_USED.compare_exchange_weak(peak, used, Ordering::SeqCst, Ordering::Relaxed) {
            Ok(_) => break,
            Err(actual) => peak = actual,
        }
    }

    current
}

/// Free memory block (stub in sequential bump allocator).
#[no_mangle]
pub extern "C" fn kfree(_ptr: *mut u8) {
    // Sequential bump allocator does not reclaim individual blocks
}

/// Get the total configured capacity of the kernel heap in bytes.
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

/// Get the allocated bytes in the kernel heap.
#[no_mangle]
pub extern "C" fn heap_get_used() -> usize {
    let start = HEAP_START.load(Ordering::SeqCst) as usize;
    let next = HEAP_NEXT.load(Ordering::SeqCst) as usize;
    if next > start {
        next - start
    } else {
        0
    }
}

/// Get the remaining unallocated free bytes in the kernel heap.
#[no_mangle]
pub extern "C" fn heap_get_free() -> usize {
    let end = HEAP_END.load(Ordering::SeqCst) as usize;
    let next = HEAP_NEXT.load(Ordering::SeqCst) as usize;
    if end > next {
        end - next
    } else {
        0
    }
}

/// Get the total number of allocation requests.
#[no_mangle]
pub extern "C" fn heap_get_alloc_count() -> usize {
    ALLOC_COUNT.load(Ordering::SeqCst)
}

/// Get peak heap memory usage in bytes.
#[no_mangle]
pub extern "C" fn heap_get_peak() -> usize {
    PEAK_USED.load(Ordering::SeqCst)
}
