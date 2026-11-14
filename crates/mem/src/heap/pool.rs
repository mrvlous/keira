// SPDX-License-Identifier: GPL-2.0-only
/*
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra <mrvlous@proton.me>
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 */

//! Segregated size-class kernel heap allocator and telemetry metrics.

pub mod stats {
    use super::allocator::{HEAP_END, HEAP_NEXT, HEAP_START};
    use core::sync::atomic::{AtomicUsize, Ordering};
    
    pub(crate) static ALLOC_COUNT: AtomicUsize = AtomicUsize::new(0);
    pub(crate) static ACTIVE_ALLOC_COUNT: AtomicUsize = AtomicUsize::new(0);
    pub(crate) static CURRENT_USED: AtomicUsize = AtomicUsize::new(0);
    pub(crate) static PEAK_USED: AtomicUsize = AtomicUsize::new(0);
    
    #[no_mangle]
    pub extern "C" fn heap_get_total() -> usize {
        let start = HEAP_START.load(Ordering::SeqCst) as usize;
        let end = HEAP_END.load(Ordering::SeqCst) as usize;
        end.saturating_sub(start)
    }
    
    #[no_mangle]
    pub extern "C" fn heap_get_used() -> usize {
        CURRENT_USED.load(Ordering::SeqCst)
    }
    
    #[no_mangle]
    pub extern "C" fn heap_get_free() -> usize {
        let total = heap_get_total();
        let used = heap_get_used();
        total.saturating_sub(used)
    }
    
    #[no_mangle]
    pub extern "C" fn heap_get_alloc_count() -> usize {
        ALLOC_COUNT.load(Ordering::SeqCst)
    }
    
    #[no_mangle]
    pub extern "C" fn heap_get_active_alloc_count() -> usize {
        ACTIVE_ALLOC_COUNT.load(Ordering::SeqCst)
    }
    
    #[no_mangle]
    pub extern "C" fn heap_get_peak() -> usize {
        PEAK_USED.load(Ordering::SeqCst)
    }
    
    #[no_mangle]
    pub extern "C" fn heap_get_arena_used() -> usize {
        let start = HEAP_START.load(Ordering::SeqCst) as usize;
        let next = HEAP_NEXT.load(Ordering::SeqCst) as usize;
        next.saturating_sub(start)
    }
    
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
    
            if !(ptr as usize).is_multiple_of(16) {
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
}

pub mod allocator {
    use super::stats::{ACTIVE_ALLOC_COUNT, ALLOC_COUNT, CURRENT_USED, PEAK_USED};
    use core::sync::atomic::{AtomicPtr, Ordering};
    use keira_core::sync::{IrqSpinLock, LockRank};
    
    pub const HEAP_ALIGNMENT: usize = 16;
    
    pub const HEAP_ALIGN_MASK: usize = HEAP_ALIGNMENT - 1;
    
    pub const BLOCK_MAGIC: u32 = 0x4B454952;
    
    pub const NUM_SIZE_CLASSES: usize = 9;
    
    pub const SIZE_CLASSES: [usize; NUM_SIZE_CLASSES] = [16, 32, 64, 128, 256, 512, 1024, 2048, 4096];
    
    pub const LARGE_CLASS: u16 = 0xFFFF;
    
    #[repr(C, align(16))]
    pub struct BlockHeader {
        pub magic: u32,
        pub size_class: u16,
        pub is_free: u16,
        pub size: usize,
        pub next_free: *mut BlockHeader,
    }
    
    pub const HEADER_SIZE: usize = core::mem::size_of::<BlockHeader>();
    
    pub(crate) static HEAP_START: AtomicPtr<u8> = AtomicPtr::new(core::ptr::null_mut());
    pub(crate) static HEAP_END: AtomicPtr<u8> = AtomicPtr::new(core::ptr::null_mut());
    pub(crate) static HEAP_NEXT: AtomicPtr<u8> = AtomicPtr::new(core::ptr::null_mut());
    
    pub(crate) static HEAP_LOCK: IrqSpinLock = IrqSpinLock::with_rank(LockRank::Heap);
    
    pub(crate) static FREE_LISTS: [AtomicPtr<BlockHeader>; NUM_SIZE_CLASSES] = [
        AtomicPtr::new(core::ptr::null_mut()),
        AtomicPtr::new(core::ptr::null_mut()),
        AtomicPtr::new(core::ptr::null_mut()),
        AtomicPtr::new(core::ptr::null_mut()),
        AtomicPtr::new(core::ptr::null_mut()),
        AtomicPtr::new(core::ptr::null_mut()),
        AtomicPtr::new(core::ptr::null_mut()),
        AtomicPtr::new(core::ptr::null_mut()),
        AtomicPtr::new(core::ptr::null_mut()),
    ];
    
    #[cfg(test)]
    pub static HEAP_TEST_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());
    
    #[inline]
    pub fn class_index_for_size(size: usize) -> Option<usize> {
        for (idx, &class_size) in SIZE_CLASSES.iter().enumerate() {
            if size <= class_size {
                return Some(idx);
            }
        }
        None
    }
    
    #[no_mangle]
    pub extern "C" fn heap_init(start: *mut u8, size: usize) {
        if start.is_null() || size < HEADER_SIZE + SIZE_CLASSES[0] {
            return;
        }
    
        let _guard = HEAP_LOCK.lock();
    
        let start_addr = (start as usize + HEAP_ALIGN_MASK) & !HEAP_ALIGN_MASK;
        let aligned_start = start_addr as *mut u8;
        let usable_size = (start as usize + size).saturating_sub(start_addr);
        let end = unsafe { aligned_start.add(usable_size) };
    
        HEAP_START.store(aligned_start, Ordering::SeqCst);
        HEAP_END.store(end, Ordering::SeqCst);
        HEAP_NEXT.store(aligned_start, Ordering::SeqCst);
        ALLOC_COUNT.store(0, Ordering::SeqCst);
        ACTIVE_ALLOC_COUNT.store(0, Ordering::SeqCst);
        CURRENT_USED.store(0, Ordering::SeqCst);
        PEAK_USED.store(0, Ordering::SeqCst);
    
        for list in &FREE_LISTS {
            list.store(core::ptr::null_mut(), Ordering::SeqCst);
        }
    }
    
    #[no_mangle]
    pub extern "C" fn kmalloc(size: usize) -> *mut u8 {
        if size == 0 || size > (usize::MAX - HEAP_ALIGN_MASK - HEADER_SIZE) {
            return core::ptr::null_mut();
        }
    
        let _guard = HEAP_LOCK.lock();
    
        let start = HEAP_START.load(Ordering::SeqCst);
        let end = HEAP_END.load(Ordering::SeqCst);
        let current = HEAP_NEXT.load(Ordering::SeqCst);
    
        if start.is_null() || current.is_null() || end.is_null() {
            return core::ptr::null_mut();
        }
    
        if let Some(class_idx) = class_index_for_size(size) {
            let class_size = SIZE_CLASSES[class_idx];
            let free_head = FREE_LISTS[class_idx].load(Ordering::SeqCst);
            if !free_head.is_null() {
                unsafe {
                    let next = (*free_head).next_free;
                    FREE_LISTS[class_idx].store(next, Ordering::SeqCst);
                    (*free_head).next_free = core::ptr::null_mut();
                    (*free_head).is_free = 0;
                    (*free_head).size = size;
    
                    let block_total = HEADER_SIZE + class_size;
                    CURRENT_USED.fetch_add(block_total, Ordering::SeqCst);
                    ALLOC_COUNT.fetch_add(1, Ordering::SeqCst);
                    ACTIVE_ALLOC_COUNT.fetch_add(1, Ordering::SeqCst);
    
                    let used = CURRENT_USED.load(Ordering::SeqCst);
                    let mut peak = PEAK_USED.load(Ordering::Relaxed);
                    while used > peak {
                        match PEAK_USED.compare_exchange_weak(
                            peak,
                            used,
                            Ordering::SeqCst,
                            Ordering::Relaxed,
                        ) {
                            Ok(_) => break,
                            Err(actual) => peak = actual,
                        }
                    }
    
                    return (free_head as *mut u8).add(HEADER_SIZE);
                }
            }
    
            let block_total = HEADER_SIZE + class_size;
            let current_addr = current as usize;
            let end_addr = end as usize;
    
            let Some(next_addr) = current_addr.checked_add(block_total) else {
                return core::ptr::null_mut();
            };
            if next_addr > end_addr {
                return core::ptr::null_mut();
            }
    
            let header = current as *mut BlockHeader;
            let new_next = next_addr as *mut u8;
            HEAP_NEXT.store(new_next, Ordering::SeqCst);
    
            unsafe {
                (*header).magic = BLOCK_MAGIC;
                (*header).size_class = class_idx as u16;
                (*header).is_free = 0;
                (*header).size = size;
                (*header).next_free = core::ptr::null_mut();
            }
    
            CURRENT_USED.fetch_add(block_total, Ordering::SeqCst);
            ALLOC_COUNT.fetch_add(1, Ordering::SeqCst);
            ACTIVE_ALLOC_COUNT.fetch_add(1, Ordering::SeqCst);
    
            let used = CURRENT_USED.load(Ordering::SeqCst);
            let mut peak = PEAK_USED.load(Ordering::Relaxed);
            while used > peak {
                match PEAK_USED.compare_exchange_weak(peak, used, Ordering::SeqCst, Ordering::Relaxed) {
                    Ok(_) => break,
                    Err(actual) => peak = actual,
                }
            }
    
            return unsafe { (header as *mut u8).add(HEADER_SIZE) };
        }
    
        let aligned_size = (size + HEAP_ALIGN_MASK) & !HEAP_ALIGN_MASK;
        let block_total = HEADER_SIZE + aligned_size;
        let current_addr = current as usize;
        let end_addr = end as usize;
    
        let Some(next_addr) = current_addr.checked_add(block_total) else {
            return core::ptr::null_mut();
        };
        if next_addr > end_addr {
            return core::ptr::null_mut();
        }
    
        let header = current as *mut BlockHeader;
        let new_next = next_addr as *mut u8;
        HEAP_NEXT.store(new_next, Ordering::SeqCst);
    
        unsafe {
            (*header).magic = BLOCK_MAGIC;
            (*header).size_class = LARGE_CLASS;
            (*header).is_free = 0;
            (*header).size = aligned_size;
            (*header).next_free = core::ptr::null_mut();
        }
    
        CURRENT_USED.fetch_add(block_total, Ordering::SeqCst);
        ALLOC_COUNT.fetch_add(1, Ordering::SeqCst);
        ACTIVE_ALLOC_COUNT.fetch_add(1, Ordering::SeqCst);
    
        let used = CURRENT_USED.load(Ordering::SeqCst);
        let mut peak = PEAK_USED.load(Ordering::Relaxed);
        while used > peak {
            match PEAK_USED.compare_exchange_weak(peak, used, Ordering::SeqCst, Ordering::Relaxed) {
                Ok(_) => break,
                Err(actual) => peak = actual,
            }
        }
    
        unsafe { (header as *mut u8).add(HEADER_SIZE) }
    }
    
    #[no_mangle]
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    pub extern "C" fn kfree(ptr: *mut u8) {
        if ptr.is_null() {
            return;
        }
    
        let _guard = HEAP_LOCK.lock();
    
        let start = HEAP_START.load(Ordering::SeqCst) as usize;
        let next = HEAP_NEXT.load(Ordering::SeqCst) as usize;
        let ptr_addr = ptr as usize;
    
        if ptr_addr < start + HEADER_SIZE || ptr_addr > next {
            return;
        }
    
        let header = unsafe { ptr.sub(HEADER_SIZE) as *mut BlockHeader };
        let header_addr = header as usize;
    
        if header_addr < start || header_addr + HEADER_SIZE > next {
            return;
        }
    
        unsafe {
            if (*header).magic != BLOCK_MAGIC {
                return;
            }
    
            if (*header).is_free != 0 {
                return;
            }
    
            (*header).is_free = 1;
            ACTIVE_ALLOC_COUNT.fetch_sub(1, Ordering::SeqCst);
    
            let size_class = (*header).size_class;
            if (size_class as usize) < NUM_SIZE_CLASSES {
                let class_idx = size_class as usize;
                let block_total = HEADER_SIZE + SIZE_CLASSES[class_idx];
                let cur = CURRENT_USED.load(Ordering::SeqCst);
                if cur >= block_total {
                    CURRENT_USED.store(cur - block_total, Ordering::SeqCst);
                } else {
                    CURRENT_USED.store(0, Ordering::SeqCst);
                }
    
                let old_head = FREE_LISTS[class_idx].load(Ordering::SeqCst);
                (*header).next_free = old_head;
                FREE_LISTS[class_idx].store(header, Ordering::SeqCst);
            } else if size_class == LARGE_CLASS {
                let block_total = HEADER_SIZE + (*header).size;
                let cur = CURRENT_USED.load(Ordering::SeqCst);
                if cur >= block_total {
                    CURRENT_USED.store(cur - block_total, Ordering::SeqCst);
                } else {
                    CURRENT_USED.store(0, Ordering::SeqCst);
                }
            }
        }
    }
}

pub use self::stats::*;
pub use self::allocator::*;
