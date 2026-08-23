// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Hardened physical page frame allocator supporting multiple memory regions and reserved zones.

use super::types::PAGE_SIZE;

const MAX_REGIONS: usize = 16;

#[derive(Clone, Copy)]
#[allow(dead_code)]
struct UsableRegion {
    start: u64,
    end: u64,
    current: u64,
}

static mut REGIONS: [UsableRegion; MAX_REGIONS] = [UsableRegion {
    start: 0,
    end: 0,
    current: 0,
}; MAX_REGIONS];
static mut REGION_COUNT: usize = 0;
static mut CURRENT_REGION_IDX: usize = 0;

static mut FREE_LIST_HEAD: u64 = 0;
static mut TOTAL_PHYS_MEM: u64 = 0;
static mut USED_FRAMES_COUNT: u64 = 0;

/// Initialize the Physical Memory Manager parsing all usable Multiboot2 memory regions.
pub unsafe fn init(multiboot_info_ptr: u64, kernel_end: u64) {
    let mut mmap_tag_ptr: u64 = 0;

    let mut addr = multiboot_info_ptr + 8;
    loop {
        let tag_type = *(addr as *const u32);
        let tag_size = *((addr + 4) as *const u32);
        if tag_type == 0 {
            break;
        }
        if tag_type == 6 {
            mmap_tag_ptr = addr;
            break;
        }
        addr += ((tag_size as u64) + 7) & !7;
    }

    if mmap_tag_ptr == 0 {
        let start = (kernel_end + PAGE_SIZE - 1) & !(PAGE_SIZE - 1);
        let size = 32 * 1024 * 1024;
        REGIONS[0] = UsableRegion {
            start,
            end: start + size,
            current: start,
        };
        REGION_COUNT = 1;
        TOTAL_PHYS_MEM = size;
        return;
    }

    let entry_size = *((mmap_tag_ptr + 8) as *const u32) as u64;
    let tag_size = *((mmap_tag_ptr + 4) as *const u32) as u64;

    let entries_start = mmap_tag_ptr + 16;
    let entries_end = mmap_tag_ptr + tag_size;

    let aligned_kernel_end = (kernel_end + PAGE_SIZE - 1) & !(PAGE_SIZE - 1);

    let mut entry_ptr = entries_start;
    while entry_ptr < entries_end {
        let base_addr = *(entry_ptr as *const u64);
        let length = *((entry_ptr + 8) as *const u64);
        let entry_type = *((entry_ptr + 16) as *const u32);

        // entry_type == 1 indicates usable RAM
        if entry_type == 1 && length >= PAGE_SIZE {
            TOTAL_PHYS_MEM += length;

            // Reserve first 1MB and kernel code/data/BSS
            let safe_start = if base_addr < aligned_kernel_end {
                aligned_kernel_end
            } else if base_addr < 0x100000 {
                0x100000
            } else {
                (base_addr + PAGE_SIZE - 1) & !(PAGE_SIZE - 1)
            };

            let seg_end = base_addr + length;
            if seg_end > safe_start && (seg_end - safe_start) >= PAGE_SIZE {
                let aligned_end = seg_end & !(PAGE_SIZE - 1);
                if REGION_COUNT < MAX_REGIONS {
                    REGIONS[REGION_COUNT] = UsableRegion {
                        start: safe_start,
                        end: aligned_end,
                        current: safe_start,
                    };
                    REGION_COUNT += 1;
                }
            }
        }

        entry_ptr += entry_size;
    }

    // Fallback if no regions parsed
    if REGION_COUNT == 0 {
        let start = aligned_kernel_end;
        let size = 32 * 1024 * 1024;
        REGIONS[0] = UsableRegion {
            start,
            end: start + size,
            current: start,
        };
        REGION_COUNT = 1;
    }
}

/// Allocate a 4KB physical page frame zero-cleared for safety.
pub fn alloc_frame() -> Option<u64> {
    unsafe {
        // 1. Check LIFO free list first
        if FREE_LIST_HEAD != 0 {
            let frame = FREE_LIST_HEAD;
            FREE_LIST_HEAD = *(frame as *const u64);
            USED_FRAMES_COUNT += 1;

            let ptr = frame as *mut u64;
            for i in 0..512 {
                *ptr.add(i) = 0;
            }
            return Some(frame);
        }

        // 2. Iterate across usable regions
        while CURRENT_REGION_IDX < REGION_COUNT {
            let region = &mut REGIONS[CURRENT_REGION_IDX];
            if region.current < region.end {
                let frame = region.current;
                region.current += PAGE_SIZE;
                USED_FRAMES_COUNT += 1;

                let ptr = frame as *mut u64;
                for i in 0..512 {
                    *ptr.add(i) = 0;
                }
                return Some(frame);
            }
            CURRENT_REGION_IDX += 1;
        }

        None
    }
}

/// Free a previously allocated physical frame and push it onto the LIFO free list.
pub fn free_frame(frame: u64) {
    if !frame.is_multiple_of(PAGE_SIZE) || frame == 0 {
        return;
    }

    unsafe {
        *(frame as *mut u64) = FREE_LIST_HEAD;
        FREE_LIST_HEAD = frame;
        if USED_FRAMES_COUNT > 0 {
            USED_FRAMES_COUNT -= 1;
        }
    }
}

/// Free multiple contiguous physical page frames.
pub fn free_contiguous_frames(start_frame: u64, count: usize) {
    for i in 0..count {
        free_frame(start_frame + (i as u64) * PAGE_SIZE);
    }
}

/// Query total detected physical memory in bytes.
pub fn total_memory() -> u64 {
    unsafe { TOTAL_PHYS_MEM }
}

/// Query currently allocated physical memory in bytes.
pub fn used_memory() -> u64 {
    unsafe { USED_FRAMES_COUNT * PAGE_SIZE }
}

/// Query currently free physical memory in bytes.
pub fn free_memory() -> u64 {
    let total = total_memory();
    let used = used_memory();
    if total > used {
        total - used
    } else {
        0
    }
}

/// Query memory statistics as a tuple (total_bytes, used_bytes, free_bytes).
pub fn get_stats() -> (u64, u64, u64) {
    (total_memory(), used_memory(), free_memory())
}
