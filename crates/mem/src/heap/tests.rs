// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Unit tests for kernel heap allocation, free-list reuse, canaries, and size classes.

use super::*;

#[test]
fn test_kmalloc_kfree_reuse_lifecycle() {
    let _lock = HEAP_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let mut buffer = [0u8; 32768];
    heap_init(buffer.as_mut_ptr(), buffer.len());

    assert_eq!(heap_get_used(), 0);
    assert_eq!(heap_get_alloc_count(), 0);
    assert_eq!(heap_get_active_alloc_count(), 0);

    let ptr1 = kmalloc(32);
    assert!(!ptr1.is_null());
    assert_eq!(ptr1 as usize % HEAP_ALIGNMENT, 0);
    assert_eq!(heap_get_alloc_count(), 1);
    assert_eq!(heap_get_active_alloc_count(), 1);
    let used_after_alloc = heap_get_used();
    assert!(used_after_alloc > 0);

    unsafe {
        core::ptr::write_bytes(ptr1, 0xAA, 32);
    }

    kfree(ptr1);
    assert_eq!(heap_get_active_alloc_count(), 0);
    assert_eq!(heap_get_used(), 0);

    let ptr2 = kmalloc(32);
    assert_eq!(ptr1, ptr2);
    assert_eq!(heap_get_alloc_count(), 2);
    assert_eq!(heap_get_active_alloc_count(), 1);
    assert_eq!(heap_get_used(), used_after_alloc);

    kfree(ptr2);
    assert_eq!(heap_get_active_alloc_count(), 0);
}

#[test]
fn test_double_free_rejection() {
    let _lock = HEAP_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let mut buffer = [0u8; 16384];
    heap_init(buffer.as_mut_ptr(), buffer.len());

    let ptr = kmalloc(64);
    assert!(!ptr.is_null());
    assert_eq!(heap_get_active_alloc_count(), 1);

    kfree(ptr);
    assert_eq!(heap_get_active_alloc_count(), 0);

    kfree(ptr);
    assert_eq!(heap_get_active_alloc_count(), 0);

    kfree(ptr);
    assert_eq!(heap_get_active_alloc_count(), 0);
}

#[test]
fn test_magic_canary_corruption_rejection() {
    let _lock = HEAP_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let mut buffer = [0u8; 16384];
    heap_init(buffer.as_mut_ptr(), buffer.len());

    let ptr = kmalloc(128);
    assert!(!ptr.is_null());
    assert_eq!(heap_get_active_alloc_count(), 1);

    let header = unsafe { ptr.sub(HEADER_SIZE) as *mut BlockHeader };
    unsafe {
        (*header).magic = 0xDEADBEEF;
    }

    kfree(ptr);
    assert_eq!(heap_get_active_alloc_count(), 1);
}

#[test]
fn test_segregated_size_classes_independence() {
    let _lock = HEAP_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let mut buffer = [0u8; 65536];
    heap_init(buffer.as_mut_ptr(), buffer.len());

    let p16 = kmalloc(16);
    let p64 = kmalloc(64);
    let p512 = kmalloc(512);
    let p4096 = kmalloc(4096);

    assert!(!p16.is_null());
    assert!(!p64.is_null());
    assert!(!p512.is_null());
    assert!(!p4096.is_null());
    assert_eq!(heap_get_active_alloc_count(), 4);

    kfree(p64);
    assert_eq!(heap_get_active_alloc_count(), 3);

    let p64_again = kmalloc(64);
    assert_eq!(p64, p64_again);
    assert_eq!(heap_get_active_alloc_count(), 4);

    kfree(p16);
    kfree(p64_again);
    kfree(p512);
    kfree(p4096);
    assert_eq!(heap_get_active_alloc_count(), 0);
    assert_eq!(heap_get_used(), 0);
}

#[test]
fn test_large_allocation_lifecycle() {
    let _lock = HEAP_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let mut buffer = [0u8; 65536];
    heap_init(buffer.as_mut_ptr(), buffer.len());

    let ptr = kmalloc(8192);
    assert!(!ptr.is_null());
    assert_eq!(ptr as usize % HEAP_ALIGNMENT, 0);
    assert_eq!(heap_get_active_alloc_count(), 1);

    unsafe {
        core::ptr::write_bytes(ptr, 0x55, 8192);
    }

    kfree(ptr);
    assert_eq!(heap_get_active_alloc_count(), 0);
    assert_eq!(heap_get_used(), 0);
}

#[test]
fn test_invalid_pointer_free_safety() {
    let _lock = HEAP_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let mut buffer = [0u8; 8192];
    heap_init(buffer.as_mut_ptr(), buffer.len());

    kfree(core::ptr::null_mut());
    let mut dummy = 42u8;
    kfree(&mut dummy as *mut u8);
    kfree(0x1234_5678 as *mut u8);
    assert_eq!(heap_get_active_alloc_count(), 0);
}
