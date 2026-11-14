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

//! Physical frame allocation bitmap, copy-on-write refcounts and frame types.

pub mod types {
    pub const PAGE_SIZE: u64 = 4096;
    
    pub const PAGE_SIZE_4K: u64 = 4096;
    
    pub const KERNEL_BASE_1MB: u64 = 0x100000;
    
    pub const MAX_PHYS_ADDR_LIMIT: u64 = 4 * 1024 * 1024 * 1024;
    
    pub const MAX_TRACKED_FRAMES: usize = (MAX_PHYS_ADDR_LIMIT / PAGE_SIZE) as usize;
    
    pub const BITMAP_WORDS: usize = MAX_TRACKED_FRAMES / 64;
    
    pub const MAX_REGIONS: usize = 16;
}

pub mod bitmap {
    use super::super::region::descriptor::{
        is_valid_ram_range, UsableRegion, CURRENT_REGION_IDX, MAX_PHYS_ADDR, REGIONS, REGION_COUNT,
    };
    use super::super::sync::PmmGuard;
    use super::types::{
        BITMAP_WORDS, KERNEL_BASE_1MB, MAX_PHYS_ADDR_LIMIT, MAX_TRACKED_FRAMES, PAGE_SIZE,
    };
    
    pub(crate) static mut FREE_LIST_HEAD: u64 = 0;
    pub(crate) static mut TOTAL_USABLE_RAM: u64 = 0;
    pub(crate) static mut USED_FRAMES_COUNT: u64 = 0;
    pub(crate) static mut FREED_FRAME_COUNT: u64 = 0;
    pub static mut ALLOCATION_BITMAP: [u64; BITMAP_WORDS] = [0; BITMAP_WORDS];
    
    pub fn alloc_frame() -> Option<u64> {
        let _guard = PmmGuard::lock();
    
        unsafe {
            #[cfg(not(test))]
            if FREE_LIST_HEAD != 0 {
                let frame = FREE_LIST_HEAD;
                FREE_LIST_HEAD = *(frame as *const u64);
                mark_frame_allocated(frame);
                USED_FRAMES_COUNT += 1;
    
                let ptr = frame as *mut u64;
                for i in 0..512 {
                    *ptr.add(i) = 0;
                }
                return Some(frame);
            }
    
            while CURRENT_REGION_IDX < REGION_COUNT {
                let region = &mut REGIONS[CURRENT_REGION_IDX];
                if region.current < region.end {
                    let frame = region.current;
                    region.current += PAGE_SIZE;
                    mark_frame_allocated(frame);
                    USED_FRAMES_COUNT += 1;
    
                    #[cfg(not(test))]
                    {
                        let ptr = frame as *mut u64;
                        for i in 0..512 {
                            *ptr.add(i) = 0;
                        }
                    }
                    return Some(frame);
                }
                CURRENT_REGION_IDX += 1;
            }
    
            None
        }
    }
    
    pub fn alloc_contiguous_frames(count: usize) -> Option<u64> {
        if count == 0 {
            return None;
        }
        if count == 1 {
            return alloc_frame();
        }
    
        let _guard = PmmGuard::lock();
    
        unsafe {
            let needed_bytes = (count as u64).checked_mul(PAGE_SIZE)?;
    
            while CURRENT_REGION_IDX < REGION_COUNT {
                let region = &mut REGIONS[CURRENT_REGION_IDX];
                if region.current.checked_add(needed_bytes)? <= region.end {
                    let start_frame = region.current;
                    region.current += needed_bytes;
    
                    for i in 0..count {
                        let frame = start_frame + (i as u64) * PAGE_SIZE;
                        mark_frame_allocated(frame);
                    }
                    USED_FRAMES_COUNT += count as u64;
    
                    #[cfg(not(test))]
                    {
                        let ptr = start_frame as *mut u64;
                        let u64_count = count * 512;
                        for i in 0..u64_count {
                            *ptr.add(i) = 0;
                        }
                    }
                    return Some(start_frame);
                }
                CURRENT_REGION_IDX += 1;
            }
    
            // Secondary fallback: linear scan across tracked frames in valid regions
            let max_search = MAX_PHYS_ADDR.min(MAX_PHYS_ADDR_LIMIT);
            let max_frames = (max_search / PAGE_SIZE) as usize;
            let mut run_start = 0usize;
            let mut run_len = 0usize;
    
            let start_idx = (KERNEL_BASE_1MB / PAGE_SIZE) as usize;
            for idx in start_idx..max_frames {
                let frame_addr = (idx as u64) * PAGE_SIZE;
                if !is_frame_allocated(frame_addr) {
                    if run_len == 0 {
                        run_start = idx;
                    }
                    run_len += 1;
                    if run_len == count {
                        let start_addr = (run_start as u64) * PAGE_SIZE;
                        if is_valid_ram_range(start_addr, needed_bytes) {
                            for i in 0..count {
                                let frame = start_addr + (i as u64) * PAGE_SIZE;
                                mark_frame_allocated(frame);
                            }
                            USED_FRAMES_COUNT += count as u64;
    
                            #[cfg(not(test))]
                            {
                                let ptr = start_addr as *mut u64;
                                let u64_count = count * 512;
                                for i in 0..u64_count {
                                    *ptr.add(i) = 0;
                                }
                            }
                            return Some(start_addr);
                        }
                        run_len = 0;
                    }
                } else {
                    run_len = 0;
                }
            }
    
            None
        }
    }
    
    pub fn alloc_order(order: u8) -> Option<u64> {
        if order > 10 {
            return None;
        }
        alloc_contiguous_frames(1usize << order)
    }
    
    pub fn is_frame_allocated(frame: u64) -> bool {
        if frame >= MAX_PHYS_ADDR_LIMIT || !frame.is_multiple_of(PAGE_SIZE) {
            return false;
        }
        let frame_idx = (frame / PAGE_SIZE) as usize;
        if frame_idx >= MAX_TRACKED_FRAMES {
            return false;
        }
        let word_idx = frame_idx / 64;
        let bit_idx = frame_idx % 64;
        unsafe { (ALLOCATION_BITMAP[word_idx] & (1u64 << bit_idx)) != 0 }
    }
    
    pub fn mark_frame_allocated(frame: u64) {
        if frame >= MAX_PHYS_ADDR_LIMIT {
            return;
        }
        let frame_idx = (frame / PAGE_SIZE) as usize;
        if frame_idx < MAX_TRACKED_FRAMES {
            let word_idx = frame_idx / 64;
            let bit_idx = frame_idx % 64;
            unsafe {
                ALLOCATION_BITMAP[word_idx] |= 1u64 << bit_idx;
            }
        }
    }
    
    pub fn mark_frame_free(frame: u64) {
        if frame >= MAX_PHYS_ADDR_LIMIT {
            return;
        }
        let frame_idx = (frame / PAGE_SIZE) as usize;
        if frame_idx < MAX_TRACKED_FRAMES {
            let word_idx = frame_idx / 64;
            let bit_idx = frame_idx % 64;
            unsafe {
                ALLOCATION_BITMAP[word_idx] &= !(1u64 << bit_idx);
            }
        }
    }
    
    pub fn get_freed_frame_count() -> u64 {
        let _guard = PmmGuard::lock();
        unsafe { FREED_FRAME_COUNT }
    }
    
    pub fn reset_pmm_stats() {
        let _guard = PmmGuard::lock();
        unsafe {
            FREE_LIST_HEAD = 0;
            USED_FRAMES_COUNT = 0;
            FREED_FRAME_COUNT = 0;
            REGION_COUNT = 0;
            CURRENT_REGION_IDX = 0;
            TOTAL_USABLE_RAM = 0;
            MAX_PHYS_ADDR = 0;
            for slot in ALLOCATION_BITMAP.iter_mut() {
                *slot = 0;
            }
        }
    }
    
    pub fn set_test_ram_region(start: u64, end: u64) {
        let _guard = PmmGuard::lock();
        unsafe {
            let capped_start = start.min(MAX_PHYS_ADDR_LIMIT);
            let capped_end = end.min(MAX_PHYS_ADDR_LIMIT);
            let size = capped_end.saturating_sub(capped_start);
            if size >= PAGE_SIZE {
                REGIONS[0] = UsableRegion {
                    start: capped_start,
                    end: capped_end,
                    current: capped_start,
                };
                REGION_COUNT = 1;
                TOTAL_USABLE_RAM = size;
                MAX_PHYS_ADDR = capped_end;
                let count = (size / PAGE_SIZE) as usize;
                USED_FRAMES_COUNT = count as u64;
                for i in 0..count {
                    mark_frame_allocated(capped_start + (i as u64) * PAGE_SIZE);
                }
            } else {
                REGION_COUNT = 0;
                TOTAL_USABLE_RAM = 0;
                MAX_PHYS_ADDR = 0;
                USED_FRAMES_COUNT = 0;
            }
        }
    }
    
    pub fn set_test_ram_region_empty(start: u64, end: u64) {
        let _guard = PmmGuard::lock();
        unsafe {
            let capped_start = start.min(MAX_PHYS_ADDR_LIMIT);
            let capped_end = end.min(MAX_PHYS_ADDR_LIMIT);
            let size = capped_end.saturating_sub(capped_start);
            if size >= PAGE_SIZE {
                REGIONS[0] = UsableRegion {
                    start: capped_start,
                    end: capped_end,
                    current: capped_start,
                };
                REGION_COUNT = 1;
                TOTAL_USABLE_RAM = size;
                MAX_PHYS_ADDR = capped_end;
                USED_FRAMES_COUNT = 0;
                for slot in ALLOCATION_BITMAP.iter_mut() {
                    *slot = 0;
                }
            } else {
                REGION_COUNT = 0;
                TOTAL_USABLE_RAM = 0;
                MAX_PHYS_ADDR = 0;
                USED_FRAMES_COUNT = 0;
            }
        }
    }
    
    pub fn free_frame(frame: u64) -> bool {
        if !frame.is_multiple_of(PAGE_SIZE)
            || frame == 0
            || !(KERNEL_BASE_1MB..MAX_PHYS_ADDR_LIMIT).contains(&frame)
        {
            return false;
        }
        let _guard = PmmGuard::lock();
        if !is_valid_ram_range(frame, PAGE_SIZE) {
            return false;
        }
        if !is_frame_allocated(frame) {
            return false;
        }
    
        unsafe {
            mark_frame_free(frame);
            FREED_FRAME_COUNT += 1;
            USED_FRAMES_COUNT = USED_FRAMES_COUNT.saturating_sub(1);
    
            #[cfg(not(test))]
            {
                *(frame as *mut u64) = FREE_LIST_HEAD;
                FREE_LIST_HEAD = frame;
            }
            true
        }
    }
    
    pub fn free_contiguous_frames(start_frame: u64, count: usize) -> bool {
        if count == 0
            || !start_frame.is_multiple_of(PAGE_SIZE)
            || !(KERNEL_BASE_1MB..MAX_PHYS_ADDR_LIMIT).contains(&start_frame)
        {
            return false;
        }
        let total_bytes = match (count as u64).checked_mul(PAGE_SIZE) {
            Some(b) => b,
            None => return false,
        };
        let end_addr = match start_frame.checked_add(total_bytes) {
            Some(e) => e,
            None => return false,
        };
        if end_addr > MAX_PHYS_ADDR_LIMIT {
            return false;
        }
    
        let _guard = PmmGuard::lock();
        if !is_valid_ram_range(start_frame, total_bytes) {
            return false;
        }
    
        for i in 0..count {
            let f = start_frame + (i as u64) * PAGE_SIZE;
            if !is_frame_allocated(f) {
                return false;
            }
        }
    
        for i in 0..count {
            let f = start_frame + (i as u64) * PAGE_SIZE;
            mark_frame_free(f);
        }
    
        unsafe {
            FREED_FRAME_COUNT += count as u64;
            if USED_FRAMES_COUNT >= count as u64 {
                USED_FRAMES_COUNT -= count as u64;
            } else {
                USED_FRAMES_COUNT = 0;
            }
    
            #[cfg(not(test))]
            {
                let end_frame = start_frame + total_bytes - PAGE_SIZE;
                let mut curr = start_frame;
                while curr < end_frame {
                    let next = curr + PAGE_SIZE;
                    *(curr as *mut u64) = next;
                    curr = next;
                }
                *(end_frame as *mut u64) = FREE_LIST_HEAD;
                FREE_LIST_HEAD = start_frame;
            }
            true
        }
    }
}

pub mod refcount {
    use core::sync::atomic::{AtomicUsize, Ordering};
    
    use super::bitmap::{free_frame, is_frame_allocated};
    use super::types::PAGE_SIZE;
    use keira_core::sync::spinlock::IrqSpinLock;
    
    pub const MAX_SHARED_COW_FRAMES: usize = 16384;
    
    const TABLE_CAPACITY: usize = MAX_SHARED_COW_FRAMES;
    const TABLE_MASK: usize = TABLE_CAPACITY - 1;
    
    #[derive(Clone, Copy, Debug)]
    struct CowFrameEntry {
        frame: u64,
        count: u32,
    }
    
    impl CowFrameEntry {
        const fn empty() -> Self {
            Self { frame: 0, count: 0 }
        }
    }
    
    static COW_TABLE_LOCK: IrqSpinLock = IrqSpinLock::new();
    static mut COW_FRAME_TABLE: [CowFrameEntry; TABLE_CAPACITY] =
        [CowFrameEntry::empty(); TABLE_CAPACITY];
    
    static COW_ACTIVE_SHARED: AtomicUsize = AtomicUsize::new(0);
    static COW_PEAK_SHARED: AtomicUsize = AtomicUsize::new(0);
    
    #[inline(always)]
    fn hash_frame(frame: u64) -> usize {
        let pfn = frame >> 12;
        let hash = pfn.wrapping_mul(0x9E37_79B9_7F4A_7C15);
        (hash as usize) & TABLE_MASK
    }
    
    #[inline(always)]
    fn probe_distance(current_idx: usize, ideal_idx: usize) -> usize {
        current_idx.wrapping_sub(ideal_idx) & TABLE_MASK
    }
    
    pub fn retain_frame(frame: u64) -> u32 {
        if frame == 0 || !frame.is_multiple_of(PAGE_SIZE) {
            return 0;
        }
    
        let _guard = COW_TABLE_LOCK.lock();
        unsafe {
            let start_idx = hash_frame(frame);
            let mut idx = start_idx;
            let mut probes = 0;
    
            while probes < TABLE_CAPACITY {
                let entry = &mut COW_FRAME_TABLE[idx];
                if entry.frame == frame {
                    entry.count = entry.count.saturating_add(1);
                    return entry.count;
                }
                if entry.frame == 0 {
                    entry.frame = frame;
                    entry.count = 2;
    
                    let active = COW_ACTIVE_SHARED.fetch_add(1, Ordering::Relaxed) + 1;
                    let mut peak = COW_PEAK_SHARED.load(Ordering::Relaxed);
                    while active > peak {
                        match COW_PEAK_SHARED.compare_exchange_weak(
                            peak,
                            active,
                            Ordering::Relaxed,
                            Ordering::Relaxed,
                        ) {
                            Ok(_) => break,
                            Err(actual) => peak = actual,
                        }
                    }
                    return 2;
                }
    
                idx = (idx + 1) & TABLE_MASK;
                probes += 1;
            }
    
            // Table capacity saturated: return 1 to indicate fallback
            1
        }
    }
    
    pub fn release_frame(frame: u64) -> bool {
        if frame == 0 || !frame.is_multiple_of(PAGE_SIZE) {
            return false;
        }
    
        {
            let _guard = COW_TABLE_LOCK.lock();
            unsafe {
                let start_idx = hash_frame(frame);
                let mut idx = start_idx;
                let mut probes = 0;
    
                while probes < TABLE_CAPACITY {
                    let entry = &mut COW_FRAME_TABLE[idx];
                    if entry.frame == 0 {
                        break;
                    }
                    if entry.frame == frame {
                        if entry.count > 2 {
                            entry.count -= 1;
                            return false;
                        }
    
                        // Count was 2, now 1: sole ownership restored, remove entry
                        entry.frame = 0;
                        entry.count = 0;
                        COW_ACTIVE_SHARED.fetch_sub(1, Ordering::Relaxed);
    
                        // Knuth backward-shift deletion to preserve linear probing invariant
                        let mut empty_idx = idx;
                        let mut next_idx = (idx + 1) & TABLE_MASK;
    
                        while COW_FRAME_TABLE[next_idx].frame != 0 {
                            let next_ideal = hash_frame(COW_FRAME_TABLE[next_idx].frame);
                            if probe_distance(next_idx, next_ideal)
                                >= probe_distance(next_idx, empty_idx)
                            {
                                COW_FRAME_TABLE[empty_idx] = COW_FRAME_TABLE[next_idx];
                                COW_FRAME_TABLE[next_idx] = CowFrameEntry::empty();
                                empty_idx = next_idx;
                            }
                            next_idx = (next_idx + 1) & TABLE_MASK;
                        }
    
                        return false;
                    }
    
                    idx = (idx + 1) & TABLE_MASK;
                    probes += 1;
                }
            }
        }
    
        free_frame(frame)
    }
    
    pub fn frame_refcount(frame: u64) -> u32 {
        if frame == 0 || !frame.is_multiple_of(PAGE_SIZE) {
            return 0;
        }
    
        let _guard = COW_TABLE_LOCK.lock();
        unsafe {
            let start_idx = hash_frame(frame);
            let mut idx = start_idx;
            let mut probes = 0;
    
            while probes < TABLE_CAPACITY {
                let entry = &COW_FRAME_TABLE[idx];
                if entry.frame == 0 {
                    break;
                }
                if entry.frame == frame {
                    return entry.count;
                }
                idx = (idx + 1) & TABLE_MASK;
                probes += 1;
            }
        }
    
        if is_frame_allocated(frame) {
            1
        } else {
            0
        }
    }
    
    pub fn active_shared_frames() -> usize {
        COW_ACTIVE_SHARED.load(Ordering::Relaxed)
    }
    
    pub fn peak_shared_frames() -> usize {
        COW_PEAK_SHARED.load(Ordering::Relaxed)
    }
    
    pub fn get_cow_refcount_stats() -> (usize, usize) {
        (active_shared_frames(), peak_shared_frames())
    }
    
    pub fn reset_refcounts() {
        let _guard = COW_TABLE_LOCK.lock();
        unsafe {
            for entry in COW_FRAME_TABLE.iter_mut() {
                *entry = CowFrameEntry::empty();
            }
        }
        COW_ACTIVE_SHARED.store(0, Ordering::Relaxed);
        COW_PEAK_SHARED.store(0, Ordering::Relaxed);
    }
}

pub use self::types::*;
pub use self::bitmap::*;
pub use self::refcount::*;
