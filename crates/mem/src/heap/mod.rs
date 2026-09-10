// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Hardened segregated free-list kernel heap allocator with size classes and active reclamation.

use core::sync::atomic::{AtomicPtr, AtomicUsize, Ordering};
use keira_core::sync::SpinLock;

/// Alignment required for all heap allocations (16 bytes).
pub const HEAP_ALIGNMENT: usize = 16;
/// Bitmask for 16-byte alignment calculations.
pub const HEAP_ALIGN_MASK: usize = HEAP_ALIGNMENT - 1;

/// Magic header canary value ("KEIR" in ASCII little-endian: 0x4B454952).
pub const BLOCK_MAGIC: u32 = 0x4B454952;

/// Number of segregated power-of-two size classes.
pub const NUM_SIZE_CLASSES: usize = 9;

/// Segregated size classes: 16B, 32B, 64B, 128B, 256B, 512B, 1024B, 2048B, 4096B.
pub const SIZE_CLASSES: [usize; NUM_SIZE_CLASSES] = [16, 32, 64, 128, 256, 512, 1024, 2048, 4096];

/// Marker for allocations larger than 4096 bytes.
pub const LARGE_CLASS: u16 = 0xFFFF;

/// Block header preceding every allocated memory block.
#[repr(C, align(16))]
pub struct BlockHeader {
    pub magic: u32,
    pub size_class: u16,
    pub is_free: u16,
    pub size: usize,
    pub next_free: *mut BlockHeader,
}

/// Size of `BlockHeader` in bytes, guaranteed to be a multiple of 16.
pub const HEADER_SIZE: usize = core::mem::size_of::<BlockHeader>();

static HEAP_START: AtomicPtr<u8> = AtomicPtr::new(core::ptr::null_mut());
static HEAP_END: AtomicPtr<u8> = AtomicPtr::new(core::ptr::null_mut());
static HEAP_NEXT: AtomicPtr<u8> = AtomicPtr::new(core::ptr::null_mut());
static ALLOC_COUNT: AtomicUsize = AtomicUsize::new(0);
static ACTIVE_ALLOC_COUNT: AtomicUsize = AtomicUsize::new(0);
static CURRENT_USED: AtomicUsize = AtomicUsize::new(0);
static PEAK_USED: AtomicUsize = AtomicUsize::new(0);

static HEAP_LOCK: SpinLock = SpinLock::new();

static FREE_LISTS: [AtomicPtr<BlockHeader>; NUM_SIZE_CLASSES] = [
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

/// Find the size class index for a given requested payload size.
#[inline]
pub fn class_index_for_size(size: usize) -> Option<usize> {
    for (idx, &class_size) in SIZE_CLASSES.iter().enumerate() {
        if size <= class_size {
            return Some(idx);
        }
    }
    None
}

/// Initialize the kernel heap allocator with a start address and size.
#[no_mangle]
pub extern "C" fn heap_init(start: *mut u8, size: usize) {
    if start.is_null() || size < HEADER_SIZE + SIZE_CLASSES[0] {
        return;
    }

    HEAP_LOCK.lock();

    let start_addr = (start as usize + HEAP_ALIGN_MASK) & !HEAP_ALIGN_MASK;
    let aligned_start = start_addr as *mut u8;
    let usable_size = if (start as usize + size) > start_addr {
        (start as usize + size) - start_addr
    } else {
        0
    };
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

    HEAP_LOCK.unlock();
}

/// Allocate a contiguous memory block from the kernel heap with 16-byte alignment.
#[no_mangle]
pub extern "C" fn kmalloc(size: usize) -> *mut u8 {
    if size == 0 || size > (usize::MAX - HEAP_ALIGN_MASK - HEADER_SIZE) {
        return core::ptr::null_mut();
    }

    HEAP_LOCK.lock();

    let start = HEAP_START.load(Ordering::SeqCst);
    let end = HEAP_END.load(Ordering::SeqCst);
    let current = HEAP_NEXT.load(Ordering::SeqCst);

    if start.is_null() || current.is_null() || end.is_null() {
        HEAP_LOCK.unlock();
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

                HEAP_LOCK.unlock();
                return (free_head as *mut u8).add(HEADER_SIZE);
            }
        }

        let block_total = HEADER_SIZE + class_size;
        let current_addr = current as usize;
        let end_addr = end as usize;

        if current_addr + block_total < current_addr || current_addr + block_total > end_addr {
            HEAP_LOCK.unlock();
            return core::ptr::null_mut();
        }

        let header = current as *mut BlockHeader;
        let new_next = (current_addr + block_total) as *mut u8;
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

        HEAP_LOCK.unlock();
        return unsafe { (header as *mut u8).add(HEADER_SIZE) };
    }

    let aligned_size = (size + HEAP_ALIGN_MASK) & !HEAP_ALIGN_MASK;
    let block_total = HEADER_SIZE + aligned_size;
    let current_addr = current as usize;
    let end_addr = end as usize;

    if current_addr + block_total < current_addr || current_addr + block_total > end_addr {
        HEAP_LOCK.unlock();
        return core::ptr::null_mut();
    }

    let header = current as *mut BlockHeader;
    let new_next = (current_addr + block_total) as *mut u8;
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

    HEAP_LOCK.unlock();
    unsafe { (header as *mut u8).add(HEADER_SIZE) }
}

/// Free a memory block previously allocated with `kmalloc`.
#[no_mangle]
pub extern "C" fn kfree(ptr: *mut u8) {
    if ptr.is_null() {
        return;
    }

    HEAP_LOCK.lock();

    let start = HEAP_START.load(Ordering::SeqCst) as usize;
    let next = HEAP_NEXT.load(Ordering::SeqCst) as usize;
    let ptr_addr = ptr as usize;

    if ptr_addr < start + HEADER_SIZE || ptr_addr > next {
        HEAP_LOCK.unlock();
        return;
    }

    let header = unsafe { ptr.sub(HEADER_SIZE) as *mut BlockHeader };
    let header_addr = header as usize;

    if header_addr < start || header_addr + HEADER_SIZE > next {
        HEAP_LOCK.unlock();
        return;
    }

    unsafe {
        if (*header).magic != BLOCK_MAGIC {
            HEAP_LOCK.unlock();
            return;
        }

        if (*header).is_free != 0 {
            HEAP_LOCK.unlock();
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

    HEAP_LOCK.unlock();
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

/// Get the currently allocated active bytes in the kernel heap.
#[no_mangle]
pub extern "C" fn heap_get_used() -> usize {
    CURRENT_USED.load(Ordering::SeqCst)
}

/// Get the remaining unallocated free bytes in the kernel heap.
#[no_mangle]
pub extern "C" fn heap_get_free() -> usize {
    let total = heap_get_total();
    let used = heap_get_used();
    total.saturating_sub(used)
}

/// Get the total number of allocation requests since initialization.
#[no_mangle]
pub extern "C" fn heap_get_alloc_count() -> usize {
    ALLOC_COUNT.load(Ordering::SeqCst)
}

/// Get the number of currently active (unfreed) memory blocks.
#[no_mangle]
pub extern "C" fn heap_get_active_alloc_count() -> usize {
    ACTIVE_ALLOC_COUNT.load(Ordering::SeqCst)
}

/// Get peak heap memory usage in bytes.
#[no_mangle]
pub extern "C" fn heap_get_peak() -> usize {
    PEAK_USED.load(Ordering::SeqCst)
}

/// Get the total bytes consumed from the underlying bump arena pool.
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

#[cfg(test)]
mod tests {
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
}
