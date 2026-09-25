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

/// Computes current kernel heap memory metrics: `(total, used, free, peak, active_allocs, fragmentation_pct)`.
pub fn heap_get_telemetry() -> (usize, usize, usize, usize, usize, usize) {
    let total = heap_get_total();
    let used = heap_get_used();
    let free = heap_get_free();
    let peak = heap_get_peak();
    let active = heap_get_active_alloc_count();
    let arena_used = heap_get_arena_used();
    let fragmentation = if arena_used > 0 && arena_used >= used {
        ((arena_used - used) * 100) / arena_used
    } else {
        0
    };
    (total, used, free, peak, active, fragmentation)
}

/// Executes an automated bare-metal stress test of the segregated free-list kernel heap allocator.
///
/// Exercises allocation, verification, non-contiguous free, free-list reuse, and total reclamation.
pub fn heap_stress_test() -> Result<(), &'static str> {
    use super::allocator::{kfree, kmalloc};

    const TEST_BLOCKS: usize = 32;
    const SIZES: [usize; 6] = [16, 32, 64, 128, 256, 512];

    let start_active = heap_get_active_alloc_count();
    let mut ptrs: [*mut u8; TEST_BLOCKS] = [core::ptr::null_mut(); TEST_BLOCKS];

    // Phase 1: Allocate mixed blocks and write verification patterns
    for i in 0..TEST_BLOCKS {
        let size = SIZES[i % SIZES.len()];
        let ptr = kmalloc(size);
        if ptr.is_null() {
            for p in ptrs.iter_mut() {
                if !p.is_null() {
                    kfree(*p);
                    *p = core::ptr::null_mut();
                }
            }
            return Err("kmalloc returned null pointer during stress test");
        }

        if (ptr as usize) % 16 != 0 {
            for p in ptrs.iter_mut() {
                if !p.is_null() {
                    kfree(*p);
                    *p = core::ptr::null_mut();
                }
            }
            return Err("Unaligned heap allocation detected");
        }

        unsafe {
            let pattern = (i as u8) ^ 0x5A;
            core::ptr::write_bytes(ptr, pattern, size);
        }
        ptrs[i] = ptr;
    }

    // Phase 2: Verify block content integrity across all allocations
    for i in 0..TEST_BLOCKS {
        let size = SIZES[i % SIZES.len()];
        let ptr = ptrs[i];
        let expected = (i as u8) ^ 0x5A;
        unsafe {
            let slice = core::slice::from_raw_parts(ptr, size);
            for &byte in slice {
                if byte != expected {
                    for p in ptrs.iter_mut() {
                        if !p.is_null() {
                            kfree(*p);
                            *p = core::ptr::null_mut();
                        }
                    }
                    return Err("Memory corruption detected during heap verification");
                }
            }
        }
    }

    // Phase 3: Free alternating blocks to exercise free-lists
    for i in (0..TEST_BLOCKS).step_by(2) {
        kfree(ptrs[i]);
        ptrs[i] = core::ptr::null_mut();
    }

    // Phase 4: Re-allocate even blocks to verify free-list reuse
    for i in (0..TEST_BLOCKS).step_by(2) {
        let size = SIZES[i % SIZES.len()];
        let ptr = kmalloc(size);
        if ptr.is_null() {
            for p in ptrs.iter_mut() {
                if !p.is_null() {
                    kfree(*p);
                    *p = core::ptr::null_mut();
                }
            }
            return Err("Free-list re-allocation failed during stress test");
        }
        ptrs[i] = ptr;
    }

    // Phase 5: Reclaim all remaining allocations
    for p in ptrs.iter_mut() {
        if !p.is_null() {
            kfree(*p);
            *p = core::ptr::null_mut();
        }
    }

    // Phase 6: Assert complete memory reclamation
    let end_active = heap_get_active_alloc_count();
    if end_active != start_active {
        return Err("Heap leak detected: active allocation count mismatch after stress test");
    }

    Ok(())
}
