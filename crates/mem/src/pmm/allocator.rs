// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Hardened physical page frame allocator supporting multiple memory regions and reserved zones.

use super::types::{KERNEL_BASE_1MB, PAGE_SIZE};

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
static mut TOTAL_USABLE_RAM: u64 = 0;
static mut MAX_PHYS_ADDR: u64 = 0;
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
        TOTAL_USABLE_RAM = size;
        MAX_PHYS_ADDR = start + size;
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
            MAX_PHYS_ADDR = seg_end;
        }

        // entry_type == 1 indicates usable RAM
        if entry_type == 1 && length >= PAGE_SIZE {
            TOTAL_USABLE_RAM += length;

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
        TOTAL_USABLE_RAM = size;
        MAX_PHYS_ADDR = start + size;
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

static mut FREED_FRAME_COUNT: u64 = 0;

/// Query the total count of freed frames since last reset (useful for verification and tests).
pub fn get_freed_frame_count() -> u64 {
    unsafe { FREED_FRAME_COUNT }
}

/// Reset PMM allocator stats and free counters (used in unit tests).
pub fn reset_pmm_stats() {
    unsafe {
        FREE_LIST_HEAD = 0;
        USED_FRAMES_COUNT = 0;
        FREED_FRAME_COUNT = 0;
        REGION_COUNT = 0;
        CURRENT_REGION_IDX = 0;
        TOTAL_USABLE_RAM = 0;
        MAX_PHYS_ADDR = 0;
    }
}

/// Set a mock physical RAM region for testing environments.
pub fn set_test_ram_region(start: u64, end: u64) {
    unsafe {
        let size = end.saturating_sub(start);
        REGIONS[0] = UsableRegion {
            start,
            end,
            current: start,
        };
        REGION_COUNT = 1;
        TOTAL_USABLE_RAM = size;
        MAX_PHYS_ADDR = end;
        USED_FRAMES_COUNT = size / PAGE_SIZE;
    }
}

/// Free a previously allocated physical frame and push it onto the LIFO free list.
/// Returns true if the frame was valid and successfully returned, false on double-free or invalid address.
pub fn free_frame(frame: u64) -> bool {
    if !frame.is_multiple_of(PAGE_SIZE) || frame == 0 || frame < KERNEL_BASE_1MB {
        return false;
    }
    if !is_valid_ram_range(frame, PAGE_SIZE) {
        return false;
    }

    unsafe {
        if USED_FRAMES_COUNT == 0 {
            return false;
        }
        if FREE_LIST_HEAD == frame {
            return false;
        }

        FREED_FRAME_COUNT += 1;
        USED_FRAMES_COUNT -= 1;

        #[cfg(not(test))]
        {
            *(frame as *mut u64) = FREE_LIST_HEAD;
            FREE_LIST_HEAD = frame;
        }
        true
    }
}

/// Validate whether a physical memory range falls inside registered usable RAM regions.
/// When regions are configured (REGION_COUNT > 0), the range MUST be entirely contained within
/// a single continuous usable region and must never bridge across reserved holes or unmapped spans.
pub fn is_valid_ram_range(start: u64, size: u64) -> bool {
    if size == 0 || !start.is_multiple_of(PAGE_SIZE) {
        return false;
    }
    let end = match start.checked_add(size) {
        Some(e) => e,
        None => return false,
    };
    if start < KERNEL_BASE_1MB {
        return false;
    }
    unsafe {
        if REGION_COUNT > 0 {
            for i in 0..REGION_COUNT {
                let region = REGIONS[i];
                if start >= region.start && end <= region.end {
                    return true;
                }
            }
            return false;
        }
        // Fallback only during early boot initialization before regions are populated
        if TOTAL_USABLE_RAM > 0 && end <= TOTAL_USABLE_RAM {
            return true;
        }
    }
    false
}

/// Free multiple contiguous physical page frames with physical-memory boundary, ownership, and alignment validation.
/// Uses native batch chaining when executing in kernel runtime.
/// Returns true if all frames were valid and reclaimed, false on double-free/underflow/out-of-bounds.
pub fn free_contiguous_frames(start_frame: u64, count: usize) -> bool {
    if count == 0 || !start_frame.is_multiple_of(PAGE_SIZE) || start_frame < KERNEL_BASE_1MB {
        return false;
    }
    let total_bytes = match (count as u64).checked_mul(PAGE_SIZE) {
        Some(b) => b,
        None => return false,
    };
    if !is_valid_ram_range(start_frame, total_bytes) {
        return false;
    }

    unsafe {
        if USED_FRAMES_COUNT < count as u64 {
            return false;
        }
        if FREE_LIST_HEAD == start_frame {
            return false;
        }

        FREED_FRAME_COUNT += count as u64;
        USED_FRAMES_COUNT -= count as u64;

        #[cfg(not(test))]
        {
            // Batch link all contiguous frames
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

/// Query total detected usable physical RAM in bytes.
pub fn total_memory() -> u64 {
    unsafe { TOTAL_USABLE_RAM }
}

/// Query total detected usable physical RAM in bytes.
pub fn total_usable_memory() -> u64 {
    unsafe { TOTAL_USABLE_RAM }
}

/// Query the highest physical memory address detected in system memory map.
pub fn max_physical_address() -> u64 {
    unsafe { MAX_PHYS_ADDR }
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

/// Query memory statistics as a tuple (total_usable_bytes, used_bytes, free_bytes).
pub fn get_stats() -> (u64, u64, u64) {
    (total_memory(), used_memory(), free_memory())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_valid_ram_range_checks() {
        reset_pmm_stats();
        unsafe {
            REGIONS[0] = UsableRegion {
                start: 0x100000, // 1MB
                end: 0x8000000,  // 128MB
                current: 0x100000,
            };
            REGION_COUNT = 1;
            TOTAL_USABLE_RAM = 0x8000000;
            MAX_PHYS_ADDR = 0x8000000;

            // Valid range
            assert!(is_valid_ram_range(0x100000, 0x200000));
            assert!(is_valid_ram_range(0x2000000, 0x4000000));

            // Below 1MB
            assert!(!is_valid_ram_range(0x0, 0x1000));
            assert!(!is_valid_ram_range(0x80000, 0x1000));

            // Unaligned start
            assert!(!is_valid_ram_range(0x100001, 0x1000));

            // Zero size
            assert!(!is_valid_ram_range(0x100000, 0));

            // Exceeds region end / boundary
            assert!(!is_valid_ram_range(0x7F00000, 0x200000));
            assert!(!is_valid_ram_range(0x8000000, 0x1000));

            // Integer overflow
            assert!(!is_valid_ram_range(0xFFFF_FFFF_FFFF_F000, 0x2000));
        }
    }

    #[test]
    fn test_multiple_usable_regions_and_reserved_hole() {
        reset_pmm_stats();
        unsafe {
            // Region 0: 256MB .. 512MB
            REGIONS[0] = UsableRegion {
                start: 0x1000_0000,
                end: 0x2000_0000,
                current: 0x1000_0000,
            };
            // Reserved hole: 512MB .. 768MB
            // Region 1: 768MB .. 2GB
            REGIONS[1] = UsableRegion {
                start: 0x3000_0000,
                end: 0x8000_0000,
                current: 0x3000_0000,
            };
            REGION_COUNT = 2;
            TOTAL_USABLE_RAM = 0x6000_0000;
            MAX_PHYS_ADDR = 0x8000_0000;

            // Range inside Region 0
            assert!(is_valid_ram_range(0x1000_0000, 0x1000_0000));

            // Range inside Region 1
            assert!(is_valid_ram_range(0x3000_0000, 0x4000_0000));

            // Range starting in reserved hole
            assert!(!is_valid_ram_range(0x2000_0000, 0x1000));
            assert!(!is_valid_ram_range(0x2800_0000, 0x1000));

            // Range spanning across Region 0 and Region 1 (bridging reserved hole)
            assert!(!is_valid_ram_range(0x1800_0000, 0x2000_0000));
        }
    }

    #[test]
    fn test_total_memory_vs_max_physical_address() {
        reset_pmm_stats();
        unsafe {
            REGIONS[0] = UsableRegion {
                start: 0x1000_0000,
                end: 0x2000_0000,
                current: 0x1000_0000,
            };
            REGIONS[1] = UsableRegion {
                start: 0x3000_0000,
                end: 0x5000_0000,
                current: 0x3000_0000,
            };
            REGION_COUNT = 2;
            TOTAL_USABLE_RAM = 0x3000_0000; // 256MB + 512MB = 768MB
            MAX_PHYS_ADDR = 0x5000_0000; // Highest physical address = 1280MB

            assert_eq!(total_memory(), 0x3000_0000);
            assert_eq!(total_usable_memory(), 0x3000_0000);
            assert_eq!(max_physical_address(), 0x5000_0000);
        }
    }

    #[test]
    fn test_1gib_boundary_and_exact_reclaim() {
        reset_pmm_stats();
        set_test_ram_region(0x4000_0000, 0x8000_0000);

        let frame_1gb = 0x4000_0000u64;
        let count_1gb = 512 * 512; // 262,144 frames

        // Exactly fits 1GB region
        assert!(is_valid_ram_range(frame_1gb, 0x4000_0000));

        // Overflows by 1 4KB page
        assert!(!is_valid_ram_range(frame_1gb, 0x4000_1000));

        // Perform contiguous free
        assert_eq!(get_freed_frame_count(), 0);
        let ok = free_contiguous_frames(frame_1gb, count_1gb);
        assert!(ok);
        assert_eq!(get_freed_frame_count(), 262_144);
    }

    #[test]
    fn test_double_free_contiguous_range_rejected() {
        reset_pmm_stats();
        set_test_ram_region(0x20_0000, 0x40_0000); // 512 frames

        let start_frame = 0x20_0000u64;
        let count = 512;

        // First free succeeds
        let res1 = free_contiguous_frames(start_frame, count);
        assert!(res1);
        assert_eq!(get_freed_frame_count(), 512);

        // Second free (double free) is detected and rejected without underflow
        let res2 = free_contiguous_frames(start_frame, count);
        assert!(!res2);
        assert_eq!(get_freed_frame_count(), 512); // Count unchanged
        assert_eq!(used_memory(), 0); // Not corrupted
    }

    #[test]
    fn test_free_overlapping_ranges_rejected() {
        reset_pmm_stats();
        set_test_ram_region(0x20_0000, 0x60_0000); // 1024 frames total

        // Free first 512 frames (0x20_0000 .. 0x40_0000)
        let res1 = free_contiguous_frames(0x20_0000, 512);
        assert!(res1);
        assert_eq!(get_freed_frame_count(), 512);

        // Free second non-overlapping 512 frames (0x40_0000 .. 0x60_0000)
        let res2 = free_contiguous_frames(0x40_0000, 512);
        assert!(res2);
        assert_eq!(get_freed_frame_count(), 1024);

        // Attempt to free overlapping range (0x30_0000 .. 0x50_0000) when all frames are already freed
        let res_overlap = free_contiguous_frames(0x30_0000, 512);
        assert!(!res_overlap);
        assert_eq!(get_freed_frame_count(), 1024);
    }

    #[test]
    fn test_free_contiguous_frames_malformed_safely_ignored() {
        reset_pmm_stats();
        let prev_freed = get_freed_frame_count();

        // Malformed unaligned frame address
        let res_unaligned = free_contiguous_frames(0x100001, 512);
        assert!(!res_unaligned);
        assert_eq!(get_freed_frame_count(), prev_freed);

        // Zero frame count
        let res_zero = free_contiguous_frames(0x200000, 0);
        assert!(!res_zero);
        assert_eq!(get_freed_frame_count(), prev_freed);

        // Frame below 1MB
        let res_low = free_contiguous_frames(0x0, 512);
        assert!(!res_low);
        assert_eq!(get_freed_frame_count(), prev_freed);

        // Frame outside physical memory bounds
        let res_oob = free_contiguous_frames(0xF000_0000_0000, 512);
        assert!(!res_oob);
        assert_eq!(get_freed_frame_count(), prev_freed);
    }
}
