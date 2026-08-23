// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Tree-based user-space memory page and page-table deallocation with strict kernel mapping preservation.

use super::mmap::cleanup_vmas_for_pml4;
use super::paging::{PAGE_HUGE, PAGE_PRESENT, PAGE_USER, PTE_ADDR_MASK};
use crate::pmm;

/// Free all user-owned mapped pages and user page-table frames from a process's PML4.
/// Walks the entire 4-level page table structure, ensuring every user physical frame is freed exactly once
/// without relying on hard-coded address ranges, while strictly preserving shared kernel mappings.
pub unsafe fn free_user_pages(pml4_phys: u64, _program_break: u64) {
    if pml4_phys == 0 {
        return;
    }

    let pml4 = pml4_phys as *const u64;

    // 1. Clean up user regions under PML4[0] (User code and low user mappings)
    let pml4_0 = *pml4;
    if (pml4_0 & PAGE_PRESENT) != 0 {
        let pdpt_phys = pml4_0 & PTE_ADDR_MASK;
        let pdpt = pdpt_phys as *const u64;

        // Traverse all entries under PDPT
        for i in 0..512 {
            // Strictly protect kernel mappings: PDPT[0] (identity map 0..1GB) and PDPT[3] (MMIO 3..4GB)
            if i == 0 || i == 3 {
                continue;
            }

            let pdpt_entry = *pdpt.add(i);
            if (pdpt_entry & PAGE_PRESENT) != 0 {
                if (pdpt_entry & PAGE_HUGE) != 0 {
                    // 1GiB Huge Page leaf frame under PML4[0]
                    if (pdpt_entry & PAGE_USER) != 0 {
                        let frame = pdpt_entry & super::paging::PTE_ADDR_MASK_1G;
                        if frame >= pmm::KERNEL_BASE_1MB && frame.is_multiple_of(0x4000_0000) {
                            pmm::free_contiguous_frames(frame, 512 * 512);
                        }
                    }
                } else {
                    free_user_page_table_subtree(pdpt_entry & PTE_ADDR_MASK, 2);
                }
            }
        }

        // Free the child process's dedicated PDPT frame if dynamically allocated
        if pdpt_phys >= pmm::KERNEL_BASE_1MB {
            pmm::free_frame(pdpt_phys);
        }
    }

    // 2. Clean up user regions under PML4[1..256] (User heap, mmap, stack in lower canonical half)
    for i in 1..256 {
        let entry = *pml4.add(i);
        if (entry & PAGE_PRESENT) != 0 {
            free_user_page_table_subtree(entry & PTE_ADDR_MASK, 3);
        }
    }

    // 3. Clean up process VMA metadata
    cleanup_vmas_for_pml4(pml4_phys);

    // 4. Free the PML4 root frame itself if dynamically allocated
    if pml4_phys >= pmm::KERNEL_BASE_1MB {
        pmm::free_frame(pml4_phys);
    }
}

/// Recursively walk a user page table tree, freeing all user-owned mapped physical page frames
/// (at level 1) and all intermediate page table frames (at levels 2 and 3).
/// Level 3 = PDPT, Level 2 = PD, Level 1 = PT
unsafe fn free_user_page_table_subtree(table_phys: u64, level: u32) {
    if table_phys < pmm::KERNEL_BASE_1MB {
        return;
    }

    let table = table_phys as *const u64;

    if level == 1 {
        // Level 1: Page Table (PT). Each present user entry is a mapped 4KB user physical frame.
        for i in 0..512 {
            let entry = *table.add(i);
            if (entry & PAGE_PRESENT) != 0 && (entry & PAGE_USER) != 0 {
                let frame = entry & PTE_ADDR_MASK;
                if frame >= pmm::KERNEL_BASE_1MB {
                    pmm::free_frame(frame);
                }
            }
        }
    } else {
        // Level 2 (PD) or Level 3 (PDPT): Walk intermediate entries
        for i in 0..512 {
            let entry = *table.add(i);
            if (entry & PAGE_PRESENT) != 0 {
                if (entry & PAGE_HUGE) != 0 {
                    // Huge page (2MB at level 2, 1GB at level 3) is a leaf frame.
                    // Reclaim underlying physical frames if dynamically allocated and owned by user.
                    if (entry & PAGE_USER) != 0 {
                        let frame = if level == 3 {
                            entry & super::paging::PTE_ADDR_MASK_1G
                        } else {
                            entry & super::paging::PTE_ADDR_MASK_2M
                        };
                        if frame >= pmm::KERNEL_BASE_1MB {
                            let count = if level == 3 {
                                512 * 512 // 1GB in 4KB frames
                            } else {
                                512 // 2MB in 4KB frames
                            };
                            pmm::free_contiguous_frames(frame, count);
                        }
                    }
                } else {
                    let child_phys = entry & PTE_ADDR_MASK;
                    if child_phys >= pmm::KERNEL_BASE_1MB {
                        free_user_page_table_subtree(child_phys, level - 1);
                    }
                }
            }
        }
    }

    // Free the page table frame itself
    pmm::free_frame(table_phys);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_free_user_pages_with_1gib_huge_mapping_under_pml4_0() {
        pmm::reset_pmm_stats();
        let mut mock_pml4 = [0u64; 512];
        let mut mock_pdpt = [0u64; 512];

        let frame_1gb = 0x4000_0000u64;
        mock_pdpt[1] = frame_1gb | PAGE_PRESENT | PAGE_USER | PAGE_HUGE;

        let pdpt_phys = mock_pdpt.as_mut_ptr() as u64;
        mock_pml4[0] = pdpt_phys | PAGE_PRESENT | PAGE_USER;

        let pml4_phys = mock_pml4.as_mut_ptr() as u64;

        // Register usable physical RAM region covering the 1GB huge page
        pmm::set_test_ram_region(0x4000_0000, 0x8000_0000);

        // Verify that 0 frames were freed before
        assert_eq!(pmm::get_freed_frame_count(), 0);

        unsafe {
            free_user_pages(pml4_phys, 0x600000000000);
        }

        // Verify that the 1GB huge page (262,144 4KB frames) was completely reclaimed
        assert_eq!(pmm::get_freed_frame_count(), 262_144);
    }
}
