// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Keira Kernel: Physical Memory Manager (PMM)
//!
//! Parses the Multiboot2 memory map to locate available RAM regions and
//! manages physical memory pages (4KB frames) using a zero-overhead free-list allocator.

use crate::io::serial;

// Constants for Page/Frame size and Kernel Memory Base
pub const PAGE_SIZE: u64 = 4096;
pub const PAGE_SIZE_4K: u64 = 4096;
pub const KERNEL_BASE_1MB: u64 = 0x100000;

// Global state for physical frame allocator
static mut FREE_MEM_START: u64 = 0;
static mut FREE_MEM_END: u64 = 0;
static mut CURRENT_PTR: u64 = 0;
static mut FREE_LIST_HEAD: u64 = 0;

static mut TOTAL_PHYS_MEM: u64 = 0;
static mut USED_FRAMES_COUNT: u64 = 0;

/// Initialize the Physical Memory Manager using Multiboot2 info pointer
pub unsafe fn init(multiboot_info_ptr: u64, kernel_end: u64) {
    let mut mmap_tag_ptr: u64 = 0;

    // Parse Multiboot2 tags to locate the Memory Map tag (type 6)
    // Skip total_size and reserved
    let mut addr = multiboot_info_ptr + 8;
    loop {
        let tag_type = *(addr as *const u32);
        let tag_size = *((addr + 4) as *const u32);
        if tag_type == 0 {
            // End tag
            break;
        }
        if tag_type == 6 {
            mmap_tag_ptr = addr;
            break;
        }
        // Align to 8 bytes
        addr += ((tag_size as u64) + 7) & !7;
    }

    if mmap_tag_ptr == 0 {
        serial::print_str("PMM Init: Warning - Multiboot2 Memory Map tag not found!\n");
        // Fallback to a default 32MB range starting at kernel_end aligned to 4KB
        let start = (kernel_end + PAGE_SIZE - 1) & !(PAGE_SIZE - 1);
        FREE_MEM_START = start;
        FREE_MEM_END = start + 32 * 1024 * 1024;
        CURRENT_PTR = start;
        TOTAL_PHYS_MEM = 32 * 1024 * 1024;
        return;
    }

    // Parse Memory Map entries
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

/// Allocate a 4KB page frame
pub fn alloc_frame() -> Option<u64> {
    unsafe {
        // 1. Try to pop from free list
        if FREE_LIST_HEAD != 0 {
            let frame = FREE_LIST_HEAD;
            // The first 8 bytes of the free frame contain the next pointer
            FREE_LIST_HEAD = *(frame as *const u64);
            USED_FRAMES_COUNT += 1;

            // Clear page to 0 for safety
            let ptr = frame as *mut u64;
            for i in 0..512 {
                *ptr.add(i) = 0;
            }
            return Some(frame);
        }

        // 2. Otherwise bump allocator pointer
        if CURRENT_PTR + PAGE_SIZE <= FREE_MEM_END {
            let frame = CURRENT_PTR;
            CURRENT_PTR += PAGE_SIZE;
            USED_FRAMES_COUNT += 1;

            // Clear page to 0 for safety
            let ptr = frame as *mut u64;
            for i in 0..512 {
                *ptr.add(i) = 0;
            }
            return Some(frame);
        }

        // Out of physical memory
        None
    }
}

/// Free an allocated page frame
pub fn free_frame(frame_addr: u64) {
    if frame_addr == 0 || frame_addr % PAGE_SIZE != 0 {
        return;
    }
    unsafe {
        // Link the freed page to the front of the free list
        let ptr = frame_addr as *mut u64;
        *ptr = FREE_LIST_HEAD;
        FREE_LIST_HEAD = frame_addr;

        if USED_FRAMES_COUNT > 0 {
            USED_FRAMES_COUNT -= 1;
        }
    }
}

/// Get physical memory statistics
pub fn get_stats() -> (u64, u64, u64) {
    unsafe {
        let total = TOTAL_PHYS_MEM;
        let used = USED_FRAMES_COUNT * PAGE_SIZE;
        let free = total.saturating_sub(used);
        (total, used, free)
    }
}

// Helpers for printing numbers to serial without std
#[allow(dead_code)]
fn print_hex(val: u64) {
    let hex_chars = b"0123456789ABCDEF";
    let mut buf = [0u8; 16];
    let mut temp = val;
    for i in (0..16).rev() {
        buf[i] = hex_chars[(temp & 0xF) as usize];
        temp >>= 4;
    }
    if let Ok(s) = core::str::from_utf8(&buf) {
        serial::print_str(s);
    }
}

#[allow(dead_code)]
fn print_decimal(val: u64) {
    let mut buf = [0u8; 20];
    let mut idx = 20;
    let mut temp = val;
    if temp == 0 {
        serial::print_str("0");
        return;
    }
    while temp > 0 {
        idx -= 1;
        buf[idx] = b'0' + (temp % 10) as u8;
        temp /= 10;
    }
    if let Ok(s) = core::str::from_utf8(&buf[idx..]) {
        serial::print_str(s);
    }
}
