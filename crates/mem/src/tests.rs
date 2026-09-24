// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Integration tests verifying cross-subsystem orchestration across PMM, VMM, Heap, Slab, and Swap.

use super::*;

#[test]
fn test_memory_subsystem_e2e_orchestration() {
    let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    reset_pmm_stats();

    // 1. Configure usable physical RAM span (32 MiB: 0x20_0000 .. 0x220_0000)
    set_test_ram_region_empty(0x20_0000, 0x220_0000);
    assert_eq!(total_usable_memory(), 0x200_0000);
    assert_eq!(used_memory(), 0);
    assert_eq!(free_memory(), 0x200_0000);

    // 2. Allocate physical frames from PMM
    let f1 = alloc_frame().expect("PMM frame 1 allocation");
    let f2 = alloc_frame().expect("PMM frame 2 allocation");
    assert_ne!(f1, f2);
    assert!(is_frame_allocated(f1));
    assert!(is_frame_allocated(f2));

    // 3. Free physical frames back to PMM
    assert!(free_frame(f1));
    assert!(free_frame(f2));
    assert!(!is_frame_allocated(f1));
    assert!(!is_frame_allocated(f2));

    // 4. Verify PMM structural invariants
    assert!(verify_pmm_invariants().is_ok());

    // 5. Test Kernel Heap sub-allocator
    let mut heap_buffer = [0u8; 16384];
    heap_init(heap_buffer.as_mut_ptr(), heap_buffer.len());
    let heap_ptr = kmalloc(128);
    assert!(!heap_ptr.is_null());
    assert_eq!(heap_get_active_alloc_count(), 1);
    kfree(heap_ptr);
    assert_eq!(heap_get_active_alloc_count(), 0);

    // 6. Test Object Cache (Slab)
    let cache = kmem_cache_create("e2e_test_cache", 64, 16);
    let obj = cache.alloc();
    assert!(!obj.is_null());
    assert_eq!(cache.allocated_count(), 1);
    cache.free(obj);
    assert_eq!(cache.allocated_count(), 0);
    cache.reap();

    // 7. Test DMA Buffer Allocation
    let dma = alloc_dma_buffer(4096).expect("DMA buffer allocation");
    assert_eq!(dma.size, 4096);
    assert!(is_frame_allocated(dma.paddr));
    assert!(free_frame(dma.paddr));

    // 8. Test Swap Pager Lifecycle
    assert!(swapon("/dev/sda2", 0).is_ok());
    assert!(swap_is_active());
    let slot = alloc_swap_slot().expect("swap slot allocation");
    assert_eq!(slot, 0);
    assert!(free_swap_slot(slot).is_ok());
    assert!(swapoff(None).is_ok());
    assert!(!swap_is_active());
}
