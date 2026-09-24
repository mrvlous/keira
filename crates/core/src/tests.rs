// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Core kernel foundation and data structure integration tests.

use super::*;
use crate::error::errno::{errno_to_ret, error_to_errno, EACCES, EFAULT, EINVAL, ENOENT, ENOMEM};

#[test]
fn test_bitmap_lifecycle_and_allocation() {
    let mut bm = Bitmap::<4>::new();
    assert_eq!(bm.total_bits(), 256);

    for i in 0..64 {
        assert!(!bm.test(i));
    }

    for expected in 0..70 {
        let alloc = bm.allocate_first_free().expect("Allocation should succeed");
        assert_eq!(alloc, expected);
        assert!(bm.test(expected));
    }

    assert!(bm.test(69));
    assert!(!bm.test(70));

    assert!(bm.clear(5));
    assert!(!bm.test(5));

    let reclaimed = bm
        .allocate_first_free()
        .expect("Reclaiming bit 5 should succeed");
    assert_eq!(reclaimed, 5);
    assert!(bm.test(5));

    assert!(!bm.set(256));
    assert!(!bm.clear(256));
    assert!(!bm.test(256));

    let mut full_bm = Bitmap::<2>::full();
    assert_eq!(full_bm.allocate_first_free(), None);
}

#[test]
fn test_ring_buffer_fifo_ordering_and_overwrite() {
    let mut rb = RingBuffer::<u32, 4>::new(0);
    assert!(rb.is_empty());
    assert_eq!(rb.len(), 0);
    assert_eq!(rb.capacity(), 4);
    assert!(!rb.is_full());

    rb.push(10);
    rb.push(20);
    rb.push(30);
    assert_eq!(rb.len(), 3);
    assert_eq!(rb.peek(), Some(10));
    assert!(!rb.is_full());

    rb.push(40);
    assert!(rb.is_full());
    assert_eq!(rb.len(), 4);

    rb.push(50);
    assert!(rb.is_full());
    assert_eq!(rb.len(), 4);
    assert_eq!(rb.peek(), Some(20));

    assert_eq!(rb.pop(), Some(20));
    assert_eq!(rb.pop(), Some(30));
    assert_eq!(rb.pop(), Some(40));
    assert_eq!(rb.pop(), Some(50));
    assert_eq!(rb.pop(), None);
    assert!(rb.is_empty());

    rb.push(100);
    rb.clear();
    assert!(rb.is_empty());
    assert_eq!(rb.pop(), None);
}

#[test]
fn test_lru_cache_lifecycle_and_eviction_policy() {
    let mut cache = LruCache::<u32, u64, 3>::new(0, 0);
    assert!(cache.is_empty());
    assert_eq!(cache.capacity(), 3);

    cache.insert(1, 100);
    cache.insert(2, 200);
    cache.insert(3, 300);
    assert_eq!(cache.len(), 3);

    assert_eq!(cache.peek(&1), Some(100));

    assert_eq!(cache.get(&1), Some(100));

    cache.insert(4, 400);

    assert_eq!(cache.get(&2), None);
    assert_eq!(cache.get(&1), Some(100));
    assert_eq!(cache.get(&3), Some(300));
    assert_eq!(cache.get(&4), Some(400));

    cache.insert(1, 101);
    assert_eq!(cache.get(&1), Some(101));

    cache.clear();
    assert!(cache.is_empty());
    assert_eq!(cache.len(), 0);
    assert_eq!(cache.get(&1), None);
}

#[test]
fn test_spin_mutex_with_ring_buffer_integration() {
    let buffer = SpinMutex::new(RingBuffer::<u8, 8>::new(0));

    {
        let mut guard = buffer.lock();
        guard.push(0xAA);
        guard.push(0xBB);
        assert_eq!(guard.len(), 2);
    }

    {
        let mut guard = buffer.lock();
        assert_eq!(guard.pop(), Some(0xAA));
        assert_eq!(guard.pop(), Some(0xBB));
        assert_eq!(guard.pop(), None);
    }
}

#[test]
fn test_irq_mutex_with_bitmap_integration() {
    let shared_bitmap = IrqMutex::with_rank(Bitmap::<2>::new(), LockRank::Pmm);

    {
        let mut guard = shared_bitmap.lock();
        let alloc = guard
            .allocate_first_free()
            .expect("Allocation should succeed");
        assert_eq!(alloc, 0);
        assert!(guard.test(0));
    }

    {
        let guard = shared_bitmap.lock();
        assert!(guard.test(0));
        assert!(!guard.test(1));
    }
}

#[test]
fn test_raw_irq_spinlock_lifecycle() {
    let lock = IrqSpinLock::with_rank(LockRank::Pmm);
    {
        let _guard = lock.lock();
    }
}

#[test]
fn test_kernel_error_and_errno_translation() {
    let err_oom = KernelError::OutOfMemory;
    assert_eq!(err_oom.as_str(), "Out of physical or virtual memory");
    assert_eq!(error_to_errno(err_oom), ENOMEM);
    assert_eq!(errno_to_ret(error_to_errno(err_oom)), -12);

    let err_inval = KernelError::InvalidArgument;
    assert_eq!(
        err_inval.as_str(),
        "Invalid parameter provided to kernel routine"
    );
    assert_eq!(error_to_errno(err_inval), EINVAL);
    assert_eq!(errno_to_ret(error_to_errno(err_inval)), -22);

    let err_perm = KernelError::PermissionDenied;
    assert_eq!(err_perm.as_str(), "Access permission denied");
    assert_eq!(error_to_errno(err_perm), EACCES);

    let err_not_found = KernelError::NotFound;
    assert_eq!(err_not_found.as_str(), "Target resource not found");
    assert_eq!(error_to_errno(err_not_found), ENOENT);

    let err_fault = KernelError::Fault;
    assert_eq!(err_fault.as_str(), "Bad address or memory fault");
    assert_eq!(error_to_errno(err_fault), EFAULT);
}

#[test]
fn test_memory_alignment_and_layout_helpers() {
    assert_eq!(PAGE_SIZE_4K, 4096);
    assert_eq!(PAGE_SIZE_2M, 2 * 1024 * 1024);
    assert_eq!(PAGE_SIZE_1G, 1024 * 1024 * 1024);

    assert_eq!(PAGE_SHIFT_4K, 12);
    assert_eq!(PAGE_SHIFT_2M, 21);
    assert_eq!(PAGE_SHIFT_1G, 30);

    assert_eq!(align_up(0x1001, PAGE_SIZE_4K), 0x2000);
    assert_eq!(align_down(0x1FFF, PAGE_SIZE_4K), 0x1000);
    assert!(is_aligned(0x2000, PAGE_SIZE_4K));
    assert!(!is_aligned(0x2001, PAGE_SIZE_4K));

    assert_eq!(align_up(MIB + 10, PAGE_SIZE_2M), 2 * MIB);
    assert_eq!(align_down(2 * MIB + 50, PAGE_SIZE_2M), 2 * MIB);
}
