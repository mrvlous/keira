// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Unit tests for hierarchical per-CPU magazine object caches and reclamation.

use super::*;
use crate::heap::{heap_init, HEAP_TEST_MUTEX};

#[test]
fn test_magazine_primitives() {
    let mut mag = Magazine::new();
    assert!(mag.is_empty());
    assert!(!mag.is_full());
    assert_eq!(mag.rounds(), 0);

    let mut dummy_objects = [0u8; MAGAZINE_CAPACITY];
    for obj in &mut dummy_objects {
        let ptr = obj as *mut u8;
        assert!(mag.push(ptr).is_ok());
    }

    assert!(mag.is_full());
    assert!(!mag.is_empty());
    assert_eq!(mag.rounds(), MAGAZINE_CAPACITY);

    let overflow_dummy = 0xAAu8;
    assert!(mag.push(overflow_dummy as *mut u8).is_err());

    // LIFO pop verification
    for i in (0..MAGAZINE_CAPACITY).rev() {
        let expected = &mut dummy_objects[i] as *mut u8;
        assert_eq!(mag.pop(), Some(expected));
    }

    assert!(mag.is_empty());
    assert_eq!(mag.pop(), None);
}

#[test]
fn test_global_depot_exchange() {
    let depot = GlobalDepot::new();
    assert_eq!(depot.full_count(), 0);
    assert_eq!(depot.empty_count(), 0);

    let empty_mag = Magazine::new();
    let res = depot.exchange_empty(empty_mag);
    assert!(res.is_none());
    assert_eq!(depot.empty_count(), 1);

    let mut full_mag = Magazine::new();
    for _ in 0..MAGAZINE_CAPACITY {
        let _ = full_mag.push(0x1000 as *mut u8);
    }

    let empty_res = depot.exchange_full(full_mag);
    assert!(empty_res.is_some());
    assert_eq!(depot.full_count(), 1);

    let another_empty = Magazine::new();
    let exchange_res = depot.exchange_empty(another_empty);
    assert!(exchange_res.is_some());
    assert_eq!(depot.full_count(), 0);

    let (drained, count) = depot.drain_full();
    assert_eq!(count, 0);
    assert!(drained[0].is_none());
}

#[test]
fn test_cpu_depot_local_operations() {
    let mut cpu_depot = CpuDepot::new();
    let global_depot = GlobalDepot::new();

    let mut dummies = [0u8; 64];

    // Alloc with refill callback
    let ptr1 = cpu_depot.alloc(&global_depot, |mag| {
        for obj in dummies.iter_mut().take(MAGAZINE_CAPACITY) {
            let _ = mag.push(obj as *mut u8);
        }
        true
    });
    assert!(ptr1.is_some());
    assert_eq!(cpu_depot.cached_count(), MAGAZINE_CAPACITY - 1);

    // Free back to active magazine
    cpu_depot.free(ptr1.unwrap(), &global_depot);
    assert_eq!(cpu_depot.cached_count(), MAGAZINE_CAPACITY);
    assert_eq!(cpu_depot.free_hits(), 1);
}

#[test]
fn test_cross_cpu_magazine_absorption() {
    let global_depot = GlobalDepot::new();
    let mut cpu0_depot = CpuDepot::new();
    let mut cpu1_depot = CpuDepot::new();

    let mut dummy_objects = [0u8; 128];
    let obj = &mut dummy_objects[0] as *mut u8;

    // CPU 0 supplies and allocates obj
    let alloced = cpu0_depot.alloc(&global_depot, |mag| mag.push(obj).is_ok());
    assert_eq!(alloced, Some(obj));
    assert_eq!(cpu0_depot.cached_count(), 0);

    // CPU 1 absorbs the free into its local magazine without locks/cross-CPU calls
    cpu1_depot.free(alloced.unwrap(), &global_depot);
    assert_eq!(cpu1_depot.cached_count(), 1);
    assert_eq!(cpu1_depot.free_hits(), 1);

    // CPU 1 can now allocate that exact object from its active magazine
    let alloced_cpu1 = cpu1_depot.alloc(&global_depot, |_| false);
    assert_eq!(alloced_cpu1, Some(obj));
    assert_eq!(cpu1_depot.cached_count(), 0);
}

#[test]
fn test_kmem_cache_lifecycle() {
    let _lock = HEAP_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let mut buffer = [0u8; 32768];
    heap_init(buffer.as_mut_ptr(), buffer.len());

    let cache = KmemCache::new("test_cache", 64, 16);
    assert_eq!(cache.allocated_count(), 0);
    assert_eq!(cache.total_count(), 0);
    assert_eq!(cache.free_count(), 0);

    let obj1 = cache.alloc();
    assert!(!obj1.is_null());
    assert_eq!(cache.allocated_count(), 1);
    assert_eq!(cache.total_count(), MAGAZINE_CAPACITY);
    assert_eq!(cache.free_count(), MAGAZINE_CAPACITY - 1);

    unsafe {
        core::ptr::write_bytes(obj1, 0x5A, 64);
    }

    cache.free(obj1);
    assert_eq!(cache.allocated_count(), 0);
    assert_eq!(cache.total_count(), MAGAZINE_CAPACITY);
    assert_eq!(cache.free_count(), MAGAZINE_CAPACITY);

    let obj2 = cache.alloc();
    assert_eq!(obj1, obj2);
    assert_eq!(cache.allocated_count(), 1);
    assert_eq!(cache.total_count(), MAGAZINE_CAPACITY);
    assert_eq!(cache.free_count(), MAGAZINE_CAPACITY - 1);

    cache.free(obj2);
    assert_eq!(cache.allocated_count(), 0);
    assert_eq!(cache.free_count(), MAGAZINE_CAPACITY);

    cache.reap();
    assert_eq!(cache.free_count(), 0);
    assert_eq!(cache.total_count(), 0);
}

#[test]
fn test_kmem_cache_batch_alloc_and_depot_exchange() {
    let _lock = HEAP_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let mut buffer = [0u8; 65536];
    heap_init(buffer.as_mut_ptr(), buffer.len());

    let cache = KmemCache::new("batch_cache", 64, 16);

    // Allocate 40 objects (crosses the 32 magazine capacity boundary)
    let mut ptrs = [core::ptr::null_mut(); 40];
    for ptr in &mut ptrs {
        *ptr = cache.alloc();
        assert!(!(*ptr).is_null());
    }
    assert_eq!(cache.allocated_count(), 40);

    // Free all 40 objects
    for &ptr in &ptrs {
        cache.free(ptr);
    }
    assert_eq!(cache.allocated_count(), 0);

    cache.reap();
    assert_eq!(cache.free_count(), 0);
    assert_eq!(cache.total_count(), 0);
}

#[test]
fn test_kmem_cache_c_abi() {
    let _lock = HEAP_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let mut buffer = [0u8; 32768];
    heap_init(buffer.as_mut_ptr(), buffer.len());

    let cache = kmem_cache_create("c_abi_cache", 32, 8);
    let cache_ptr = &cache as *const KmemCache;

    unsafe {
        assert!(kmem_cache_alloc(core::ptr::null()).is_null());
        kmem_cache_free(core::ptr::null(), core::ptr::null_mut());
        kmem_cache_free(cache_ptr, core::ptr::null_mut());

        let obj = kmem_cache_alloc(cache_ptr);
        assert!(!obj.is_null());
        assert_eq!(cache.allocated_count(), 1);

        kmem_cache_free(cache_ptr, obj);
        assert_eq!(cache.allocated_count(), 0);
    }
}

#[test]
fn test_static_caches_definition() {
    assert_eq!(TASK_CACHE.name(), "task_struct");
    assert_eq!(TASK_CACHE.obj_size(), 512);
    assert_eq!(INODE_CACHE.name(), "inode_cache");
    assert_eq!(INODE_CACHE.obj_size(), 256);
    assert_eq!(FD_CACHE.name(), "file_desc_cache");
    assert_eq!(FD_CACHE.obj_size(), 64);
    assert_eq!(VMA_CACHE.name(), "vma_cache");
    assert_eq!(VMA_CACHE.obj_size(), 128);
}
