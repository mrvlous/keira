// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Bootloader Multiboot2 memory map parsing and early allocator initialization.

use super::super::frame::bitmap::TOTAL_USABLE_RAM;
use super::super::frame::types::{MAX_PHYS_ADDR_LIMIT, MAX_REGIONS, PAGE_SIZE};
use super::super::sync::PmmGuard;
use super::descriptor::{UsableRegion, MAX_PHYS_ADDR, REGIONS, REGION_COUNT};

/// Initializes the Physical Memory Manager by parsing Multiboot2 memory map tags.
///
/// Discovers usable physical memory segments, reserves low memory (< 1 MiB) and kernel
/// binary boundaries, and seeds the physical page frame allocator.
///
/// # Safety
///
/// The caller must ensure that `multiboot_info_ptr` points to a valid, intact Multiboot2
/// information structure provided by the bootloader, and `kernel_end` accurately designates
/// the uppermost byte occupied by the kernel ELF segments, modules, and early heap.
pub unsafe fn init(multiboot_info_ptr: u64, kernel_end: u64) {
    let _guard = PmmGuard::lock();
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
        let capped_end = (start + size).min(MAX_PHYS_ADDR_LIMIT);
        REGIONS[0] = UsableRegion {
            start,
            end: capped_end,
            current: start,
        };
        REGION_COUNT = 1;
        TOTAL_USABLE_RAM = capped_end.saturating_sub(start);
        MAX_PHYS_ADDR = capped_end;
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

        let seg_end = base_addr.saturating_add(length);
        if seg_end > MAX_PHYS_ADDR {
            MAX_PHYS_ADDR = seg_end.min(MAX_PHYS_ADDR_LIMIT);
        }

        if entry_type == 1 && length >= PAGE_SIZE && base_addr < MAX_PHYS_ADDR_LIMIT {
            let capped_seg_end = seg_end.min(MAX_PHYS_ADDR_LIMIT);

            let safe_start = if base_addr < aligned_kernel_end {
                aligned_kernel_end
            } else if base_addr < 0x100000 {
                0x100000
            } else {
                (base_addr + PAGE_SIZE - 1) & !(PAGE_SIZE - 1)
            };

            if capped_seg_end > safe_start && (capped_seg_end - safe_start) >= PAGE_SIZE {
                let aligned_end = capped_seg_end & !(PAGE_SIZE - 1);
                TOTAL_USABLE_RAM += aligned_end - safe_start;
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
}
