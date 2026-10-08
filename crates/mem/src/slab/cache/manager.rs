// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Hierarchical Per-CPU Magazine-Style Slab Object Allocator.
//!
//! Implements Jeff Bonwick's Magazine-layer architecture combined with
//! Epoch-Based Reclamation (EBR) to provide $O(1)$ zero-lock, zero-atomic
//! allocation and deallocation for fixed-size kernel descriptors.

use crate::heap::{kfree, kmalloc};
use crate::slab::magazine::{CpuDepot, GlobalDepot, Magazine, MAGAZINE_CAPACITY, MAX_CPU_CORES};
use core::sync::atomic::{AtomicPtr, AtomicUsize, Ordering};
use keira_core::sync::{IrqSpinLock, LockRank};

/// Hierarchical per-CPU magazine object cache for fixed-size kernel descriptors.
pub struct KmemCache {
    name: &'static str,
    obj_size: usize,
    align: usize,
    cpu_depots: [CpuDepot; MAX_CPU_CORES],
    cpu_locks: [IrqSpinLock; MAX_CPU_CORES],
    global_depot: GlobalDepot,
    slab_lock: IrqSpinLock,
    slab_free_list: AtomicPtr<u8>,
    allocated_count: AtomicUsize,
    total_count: AtomicUsize,
}

unsafe impl Send for KmemCache {}
unsafe impl Sync for KmemCache {}

impl KmemCache {
    /// Constructs a new hierarchical magazine object cache for descriptors of `obj_size` bytes.
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
            cpu_depots: [const { CpuDepot::new() }; MAX_CPU_CORES],
            cpu_locks: [const { IrqSpinLock::with_rank(LockRank::Heap) }; MAX_CPU_CORES],
            global_depot: GlobalDepot::new(),
            slab_lock: IrqSpinLock::with_rank(LockRank::Heap),
            slab_free_list: AtomicPtr::new(core::ptr::null_mut()),
            allocated_count: AtomicUsize::new(0),
            total_count: AtomicUsize::new(0),
        }
    }

    /// Allocates an object from the cache, prioritizing local per-CPU magazines.
    ///
    /// 1. Fast Path: Pops from local CPU active magazine ($O(1)$, zero cross-CPU contention).
    /// 2. Medium Path: Exchanges empty backup magazine with Central Global Depot.
    /// 3. Slow Path: Carves fresh objects from slab backend into active magazine.
    pub fn alloc(&self) -> *mut u8 {
        let core_id = keira_arch::cpu::get_current_core_id() % MAX_CPU_CORES;
        let cache_ptr = self as *const Self as *mut Self;

        // 1. Lock only the local core's depot (zero contention across different cores)
        {
            let _guard = self.cpu_locks[core_id].lock();
            let depot = unsafe { &mut (*cache_ptr).cpu_depots[core_id] };

            if let Some(obj) = depot.alloc(&self.global_depot, |mag| self.refill_magazine(mag)) {
                self.allocated_count.fetch_add(1, Ordering::Relaxed);
                return obj;
            }
        }

        // 2. Direct fallback if magazine and slab refill failed
        let obj = kmalloc(self.obj_size);
        if !obj.is_null() {
            self.allocated_count.fetch_add(1, Ordering::Relaxed);
            self.total_count.fetch_add(1, Ordering::Relaxed);
        }
        obj
    }

    /// Returns an allocated object back to the local per-CPU magazine cache.
    ///
    /// Transparently handles cross-CPU deallocation: the object is absorbed
    /// into the freeing CPU's local magazine without remote locking.
    pub fn free(&self, ptr: *mut u8) {
        if ptr.is_null() {
            return;
        }

        let core_id = keira_arch::cpu::get_current_core_id() % MAX_CPU_CORES;
        let cache_ptr = self as *const Self as *mut Self;

        {
            let _guard = self.cpu_locks[core_id].lock();
            let depot = unsafe { &mut (*cache_ptr).cpu_depots[core_id] };
            depot.free(ptr, &self.global_depot);
        }

        self.allocated_count.fetch_sub(1, Ordering::Relaxed);
    }

    /// Refills a magazine from the slab backend when all depots are empty.
    fn refill_magazine(&self, mag: &mut Magazine) -> bool {
        let _guard = self.slab_lock.lock();
        let _ebr = keira_core::sync::ebr::pin();

        // 1. Check pre-carved objects in slab_free_list
        let mut head = self.slab_free_list.load(Ordering::Relaxed);
        while !head.is_null() && !mag.is_full() {
            unsafe {
                let next = *(head as *mut *mut u8);
                let _ = mag.push(head);
                head = next;
            }
        }
        self.slab_free_list.store(head, Ordering::Relaxed);

        // 2. If magazine still needs objects, allocate a batch from kernel heap
        let needed = MAGAZINE_CAPACITY.saturating_sub(mag.rounds());
        if needed > 0 {
            let aligned = (self.obj_size + self.align - 1) & !(self.align - 1);
            for _ in 0..needed {
                let obj = kmalloc(aligned);
                if obj.is_null() {
                    break;
                }
                self.total_count.fetch_add(1, Ordering::Relaxed);
                let _ = mag.push(obj);
            }
        }

        !mag.is_empty()
    }

    /// Frees all currently cached, unused objects back to the kernel heap.
    pub fn reap(&self) {
        let _ebr = keira_core::sync::ebr::pin();
        let cache_ptr = self as *const Self as *mut Self;

        // 1. Drain all CPU depots
        for core_id in 0..MAX_CPU_CORES {
            let _guard = self.cpu_locks[core_id].lock();
            let (mut act, mut bkp) = unsafe { (*cache_ptr).cpu_depots[core_id].drain() };
            while let Some(ptr) = act.pop() {
                kfree(ptr);
                self.total_count.fetch_sub(1, Ordering::Relaxed);
            }
            while let Some(ptr) = bkp.pop() {
                kfree(ptr);
                self.total_count.fetch_sub(1, Ordering::Relaxed);
            }
        }

        // 2. Drain global depot
        let (full_mags, count) = self.global_depot.drain_full();
        for mut mag in full_mags.into_iter().take(count).flatten() {
            while let Some(ptr) = mag.pop() {
                kfree(ptr);
                self.total_count.fetch_sub(1, Ordering::Relaxed);
            }
        }

        // 3. Drain slab free list
        let mut head = {
            let _guard = self.slab_lock.lock();
            let h = self.slab_free_list.load(Ordering::Relaxed);
            self.slab_free_list
                .store(core::ptr::null_mut(), Ordering::Relaxed);
            h
        };

        while !head.is_null() {
            let next = unsafe { *(head as *mut *mut u8) };
            kfree(head);
            self.total_count.fetch_sub(1, Ordering::Relaxed);
            head = next;
        }
    }

    /// Retrieves the cache name string.
    #[inline]
    pub fn name(&self) -> &'static str {
        self.name
    }

    /// Retrieves the cached object size in bytes.
    #[inline]
    pub fn obj_size(&self) -> usize {
        self.obj_size
    }

    /// Retrieves the object alignment in bytes.
    #[inline]
    pub fn align(&self) -> usize {
        self.align
    }

    /// Retrieves the count of currently allocated (active) objects.
    #[inline]
    pub fn allocated_count(&self) -> usize {
        self.allocated_count.load(Ordering::Relaxed)
    }

    /// Retrieves the total number of objects created by this cache.
    #[inline]
    pub fn total_count(&self) -> usize {
        self.total_count.load(Ordering::Relaxed)
    }

    /// Retrieves the number of cached free objects ready for immediate reuse.
    #[inline]
    pub fn free_count(&self) -> usize {
        self.total_count().saturating_sub(self.allocated_count())
    }

    /// Retrieves total local magazine allocation hits across all cores.
    pub fn total_alloc_hits(&self) -> usize {
        self.cpu_depots.iter().map(|d| d.alloc_hits()).sum()
    }

    /// Retrieves total local magazine free hits across all cores.
    pub fn total_free_hits(&self) -> usize {
        self.cpu_depots.iter().map(|d| d.free_hits()).sum()
    }

    /// Retrieves total depot exchanges across all cores.
    pub fn total_exchanges(&self) -> usize {
        self.cpu_depots.iter().map(|d| d.exchanges()).sum()
    }

    /// Retrieves count of objects currently cached across per-CPU magazines.
    pub fn total_cpu_cached(&self) -> usize {
        self.cpu_depots.iter().map(|d| d.cached_count()).sum()
    }

    /// Retrieves count of objects currently cached in the central global depot.
    pub fn total_depot_cached(&self) -> usize {
        self.global_depot.full_count() * MAGAZINE_CAPACITY
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

/// Creates a new kernel object cache dynamically.
pub fn kmem_cache_create(name: &'static str, obj_size: usize, align: usize) -> KmemCache {
    KmemCache::new(name, obj_size, align)
}

/// Allocates an object from a cache via C ABI.
///
/// # Safety
/// Dereferences the raw `cache` pointer.
#[no_mangle]
pub unsafe extern "C" fn kmem_cache_alloc(cache: *const KmemCache) -> *mut u8 {
    if cache.is_null() {
        return core::ptr::null_mut();
    }
    (*cache).alloc()
}

/// Frees an object back to a cache via C ABI.
///
/// # Safety
/// Dereferences the raw `cache` and `ptr` pointers.
#[no_mangle]
pub unsafe extern "C" fn kmem_cache_free(cache: *const KmemCache, ptr: *mut u8) {
    if cache.is_null() || ptr.is_null() {
        return;
    }
    (*cache).free(ptr)
}
