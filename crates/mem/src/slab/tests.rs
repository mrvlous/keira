// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Unit tests for fixed-size descriptor object caches and reclamation.

use super::*;
use crate::heap::{heap_init, HEAP_TEST_MUTEX};

#[test]
fn test_kmem_cache_lifecycle() {
    let _lock = HEAP_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let mut buffer = [0u8; 16384];
    heap_init(buffer.as_mut_ptr(), buffer.len());

    let cache = KmemCache::new("test_cache", 64, 16);
    assert_eq!(cache.allocated_count(), 0);
    assert_eq!(cache.total_count(), 0);
    assert_eq!(cache.free_count(), 0);

    let obj1 = cache.alloc();
    assert!(!obj1.is_null());
    assert_eq!(cache.allocated_count(), 1);
    assert_eq!(cache.total_count(), 1);
    assert_eq!(cache.free_count(), 0);

    unsafe {
        core::ptr::write_bytes(obj1, 0x5A, 64);
    }

    cache.free(obj1);
    assert_eq!(cache.allocated_count(), 0);
    assert_eq!(cache.total_count(), 1);
    assert_eq!(cache.free_count(), 1);

    let obj2 = cache.alloc();
    assert_eq!(obj1, obj2);
    assert_eq!(cache.allocated_count(), 1);
    assert_eq!(cache.total_count(), 1);
    assert_eq!(cache.free_count(), 0);

    cache.free(obj2);
    assert_eq!(cache.allocated_count(), 0);
    assert_eq!(cache.free_count(), 1);

    cache.reap();
    assert_eq!(cache.free_count(), 0);
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
