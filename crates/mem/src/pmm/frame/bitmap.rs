// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Core bitmap-backed frame allocation, deallocation, and state querying.

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

/// Allocates a 4 KiB physical page frame, zero-clearing its contents for safety.
///
/// Returns the physical base address of the allocated frame on success, or `None`
/// if physical memory is exhausted.
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

/// Checks whether a physical frame is currently marked as allocated in the PMM bitmap.
pub fn is_frame_allocated(frame: u64) -> bool {
    if frame >= MAX_PHYS_ADDR_LIMIT || (frame % PAGE_SIZE) != 0 {
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

/// Marks a physical page frame as allocated in the PMM bitmap.
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

/// Marks a physical page frame as free in the PMM bitmap.
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

/// Retrieves the total count of freed frames since last reset.
pub fn get_freed_frame_count() -> u64 {
    let _guard = PmmGuard::lock();
    unsafe { FREED_FRAME_COUNT }
}

/// Resets physical memory allocator statistics, bitmap, and regions (for testing purposes).
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
        for i in 0..BITMAP_WORDS {
            ALLOCATION_BITMAP[i] = 0;
        }
    }
}

/// Configures an active physical RAM region where all frames start marked as allocated.
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

/// Configures an active physical RAM region where all frames start marked as free.
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
            for i in 0..BITMAP_WORDS {
                ALLOCATION_BITMAP[i] = 0;
            }
        } else {
            REGION_COUNT = 0;
            TOTAL_USABLE_RAM = 0;
            MAX_PHYS_ADDR = 0;
            USED_FRAMES_COUNT = 0;
        }
    }
}

/// Frees an allocated physical frame and returns it to the free pool.
///
/// Returns `true` on successful deallocation, or `false` on double-free or invalid addresses.
pub fn free_frame(frame: u64) -> bool {
    if (frame % PAGE_SIZE) != 0
        || frame == 0
        || frame < KERNEL_BASE_1MB
        || frame >= MAX_PHYS_ADDR_LIMIT
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
        if USED_FRAMES_COUNT > 0 {
            USED_FRAMES_COUNT -= 1;
        }

        #[cfg(not(test))]
        {
            *(frame as *mut u64) = FREE_LIST_HEAD;
            FREE_LIST_HEAD = frame;
        }
        true
    }
}

/// Frees multiple contiguous physical page frames with boundary and ownership validation.
///
/// Returns `true` if all frames in the range were valid and reclaimed, or `false`
/// if any frame was already free, out of bounds, or invalid.
pub fn free_contiguous_frames(start_frame: u64, count: usize) -> bool {
    if count == 0
        || (start_frame % PAGE_SIZE) != 0
        || start_frame < KERNEL_BASE_1MB
        || start_frame >= MAX_PHYS_ADDR_LIMIT
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
