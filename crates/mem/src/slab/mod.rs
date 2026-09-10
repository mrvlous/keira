// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Kernel object cache (slab-like allocator) for fixed-size kernel descriptors.

use crate::heap::{kfree, kmalloc};
use core::sync::atomic::{AtomicPtr, AtomicUsize, Ordering};
use keira_core::sync::SpinLock;

/// Kernel object cache for fixed-size, frequently allocated descriptors.
pub struct KmemCache {
    name: &'static str,
    obj_size: usize,
    align: usize,
    free_list: AtomicPtr<u8>,
    allocated_count: AtomicUsize,
    total_count: AtomicUsize,
    lock: SpinLock,
}

unsafe impl Send for KmemCache {}
unsafe impl Sync for KmemCache {}

impl KmemCache {
    /// Create a new kernel object cache for descriptors of `obj_size` bytes.
    pub const fn new(name: &'static str, obj_size: usize, align: usize) -> Self {
        let min_size = core::mem::size_of::<*mut u8>();
        let size = if obj_size < min_size {
            min_size
        } else {
            obj_size
        };
        Self {
            name,
            obj_size: size,
            align,
            free_list: AtomicPtr::new(core::ptr::null_mut()),
            allocated_count: AtomicUsize::new(0),
            total_count: AtomicUsize::new(0),
            lock: SpinLock::new(),
        }
    }

    /// Allocate an object from the cache, reusing freed instances when available.
    pub fn alloc(&self) -> *mut u8 {
        self.lock.lock();
        let head = self.free_list.load(Ordering::SeqCst);
        if !head.is_null() {
            unsafe {
                let next = *(head as *mut *mut u8);
                self.free_list.store(next, Ordering::SeqCst);
            }
            self.allocated_count.fetch_add(1, Ordering::SeqCst);
            self.lock.unlock();
            return head;
        }
        self.lock.unlock();

        let ptr = kmalloc(self.obj_size);
        if !ptr.is_null() {
            self.allocated_count.fetch_add(1, Ordering::SeqCst);
            self.total_count.fetch_add(1, Ordering::SeqCst);
        }
        ptr
    }

    /// Return an allocated object back to the cache's free list.
    pub fn free(&self, ptr: *mut u8) {
        if ptr.is_null() {
            return;
        }

        self.lock.lock();
        unsafe {
            let current_head = self.free_list.load(Ordering::SeqCst);
            *(ptr as *mut *mut u8) = current_head;
            self.free_list.store(ptr, Ordering::SeqCst);
        }
        self.allocated_count.fetch_sub(1, Ordering::SeqCst);
        self.lock.unlock();
    }

    /// Free all currently cached, unused objects back to the kernel heap.
    pub fn reap(&self) {
        self.lock.lock();
        let mut head = self.free_list.load(Ordering::SeqCst);
        self.free_list
            .store(core::ptr::null_mut(), Ordering::SeqCst);
        self.lock.unlock();

        while !head.is_null() {
            let next = unsafe { *(head as *mut *mut u8) };
            kfree(head);
            self.total_count.fetch_sub(1, Ordering::SeqCst);
            head = next;
        }
    }

    /// Get the cache name.
    #[inline]
    pub fn name(&self) -> &'static str {
        self.name
    }

    /// Get the object size in bytes.
    #[inline]
    pub fn obj_size(&self) -> usize {
        self.obj_size
    }

    /// Get the alignment in bytes.
    #[inline]
    pub fn align(&self) -> usize {
        self.align
    }

    /// Get the number of currently allocated (active) objects.
    #[inline]
    pub fn allocated_count(&self) -> usize {
        self.allocated_count.load(Ordering::SeqCst)
    }

    /// Get the total number of objects created by this cache.
    #[inline]
    pub fn total_count(&self) -> usize {
        self.total_count.load(Ordering::SeqCst)
    }

    /// Get the number of cached free objects ready for immediate reuse.
    #[inline]
    pub fn free_count(&self) -> usize {
        self.total_count().saturating_sub(self.allocated_count())
    }
}

/// Pre-allocated cache for task control blocks (512 bytes).
pub static TASK_CACHE: KmemCache = KmemCache::new("task_struct", 512, 16);

/// Pre-allocated cache for VFS inodes (256 bytes).
pub static INODE_CACHE: KmemCache = KmemCache::new("inode_cache", 256, 16);

/// Pre-allocated cache for file descriptors (64 bytes).
pub static FD_CACHE: KmemCache = KmemCache::new("file_desc_cache", 64, 16);

/// Pre-allocated cache for virtual memory area descriptors (128 bytes).
pub static VMA_CACHE: KmemCache = KmemCache::new("vma_cache", 128, 16);

/// Create a new kernel object cache dynamically.
pub fn kmem_cache_create(name: &'static str, obj_size: usize, align: usize) -> KmemCache {
    KmemCache::new(name, obj_size, align)
}

/// Allocate an object from a cache via C ABI.
#[no_mangle]
pub extern "C" fn kmem_cache_alloc(cache: *const KmemCache) -> *mut u8 {
    if cache.is_null() {
        return core::ptr::null_mut();
    }
    unsafe { (*cache).alloc() }
}

/// Free an object back to a cache via C ABI.
#[no_mangle]
pub extern "C" fn kmem_cache_free(cache: *const KmemCache, ptr: *mut u8) {
    if cache.is_null() || ptr.is_null() {
        return;
    }
    unsafe { (*cache).free(ptr) }
}

#[cfg(test)]
mod tests {
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
}
