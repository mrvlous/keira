// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! 4KB physical page frame allocator with O(1) LIFO free list and bump fallback.

use super::types::PAGE_SIZE;

static mut FREE_MEM_START: u64 = 0;
static mut FREE_MEM_END: u64 = 0;
static mut CURRENT_PTR: u64 = 0;
static mut FREE_LIST_HEAD: u64 = 0;

static mut TOTAL_PHYS_MEM: u64 = 0;
static mut USED_FRAMES_COUNT: u64 = 0;

/// Initialize the Physical Memory Manager using Multiboot2 info pointer.
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
        FREE_MEM_START = start;
        FREE_MEM_END = start + 32 * 1024 * 1024;
        CURRENT_PTR = start;
        TOTAL_PHYS_MEM = 32 * 1024 * 1024;
        return;
    }

    let entry_size = *((mmap_tag_ptr + 8) as *const u32) as u64;
    let tag_size = *((mmap_tag_ptr + 4) as *const u32) as u64;

    let entries_start = mmap_tag_ptr + 16;
    let entries_end = mmap_tag_ptr + tag_size;

    let mut largest_ram_start: u64 = 0;
    let mut largest_ram_size: u64 = 0;

    let mut entry_ptr = entries_start;
    while entry_ptr < entries_end {
        let base_addr = *(entry_ptr as *const u64);
        let length = *((entry_ptr + 8) as *const u64);
        let entry_type = *((entry_ptr + 16) as *const u32);

        if entry_type == 1 {
            TOTAL_PHYS_MEM += length;

            let safe_start = if base_addr < kernel_end {
                (kernel_end + PAGE_SIZE - 1) & !(PAGE_SIZE - 1)
            } else {
                base_addr
            };

            if base_addr + length > safe_start {
                let safe_len = (base_addr + length) - safe_start;
                if safe_len > largest_ram_size {
                    largest_ram_size = safe_len;
                    largest_ram_start = safe_start;
                }
            }
        }

        entry_ptr += entry_size;
    }

    if largest_ram_size > 0 {
        FREE_MEM_START = largest_ram_start;
        FREE_MEM_END = largest_ram_start + largest_ram_size;
        CURRENT_PTR = largest_ram_start;
    } else {
        let start = (kernel_end + PAGE_SIZE - 1) & !(PAGE_SIZE - 1);
        FREE_MEM_START = start;
        FREE_MEM_END = start + 32 * 1024 * 1024;
        CURRENT_PTR = start;
    }
}

/// Allocate a 4KB physical page frame zero-cleared for safety.
pub fn alloc_frame() -> Option<u64> {
    unsafe {
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

        if CURRENT_PTR + PAGE_SIZE <= FREE_MEM_END {
            let frame = CURRENT_PTR;
            CURRENT_PTR += PAGE_SIZE;
            USED_FRAMES_COUNT += 1;

            let ptr = frame as *mut u64;
            for i in 0..512 {
                *ptr.add(i) = 0;
            }
            return Some(frame);
        }

        None
    }
}

/// Free an allocated physical page frame back to the LIFO free list.
pub fn free_frame(frame_addr: u64) {
    if frame_addr == 0 || !frame_addr.is_multiple_of(PAGE_SIZE) {
        return;
    }
    unsafe {
        let ptr = frame_addr as *mut u64;
        *ptr = FREE_LIST_HEAD;
        FREE_LIST_HEAD = frame_addr;

        USED_FRAMES_COUNT = USED_FRAMES_COUNT.saturating_sub(1);
    }
}

/// Get physical memory statistics: (total_bytes, used_bytes, free_bytes).
pub fn get_stats() -> (u64, u64, u64) {
    unsafe {
        let total = TOTAL_PHYS_MEM;
        let used = USED_FRAMES_COUNT * PAGE_SIZE;
        let free = total.saturating_sub(used);
        (total, used, free)
    }
}
