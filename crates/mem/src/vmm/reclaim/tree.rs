// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Tree-based user-space memory page and page-table deallocation.

use super::super::area::cleanup_vmas_for_pml4;
use super::super::table::{
    PAGE_HUGE, PAGE_PRESENT, PAGE_USER, PTE_ADDR_MASK, PTE_ADDR_MASK_1G, PTE_ADDR_MASK_2M,
};
use crate::pmm;

#[inline]
fn is_valid_page_table_frame(phys: u64) -> bool {
    if (phys % pmm::PAGE_SIZE) != 0 || phys == 0 {
        return false;
    }
    #[cfg(test)]
    {
        true
    }
    #[cfg(not(test))]
    {
        phys >= pmm::KERNEL_BASE_1MB
            && phys < pmm::MAX_PHYS_ADDR_LIMIT
            && pmm::is_valid_ram_range(phys, pmm::PAGE_SIZE)
    }
}

/// Frees all user-owned mapped pages and intermediate page-table frames from a PML4 table.
///
/// Recursively walks lower-half page table trees, reclaiming individual physical frames
/// and intermediate table frames while strictly preserving shared kernel mappings.
///
/// # Safety
///
/// Deallocates physical memory frames and clears active page table pointers.
pub unsafe fn free_user_pages(pml4_phys: u64, _program_break: u64) {
    if !is_valid_page_table_frame(pml4_phys) {
        return;
    }

    let pml4 = pml4_phys as *const u64;

    let pml4_0 = *pml4;
    if (pml4_0 & PAGE_PRESENT) != 0 {
        let pdpt_phys = pml4_0 & PTE_ADDR_MASK;
        if is_valid_page_table_frame(pdpt_phys) {
            let pdpt = pdpt_phys as *const u64;

            for i in 0..512 {
                if i == 0 || i == 3 {
                    continue;
                }

                let pdpt_entry = *pdpt.add(i);
                if (pdpt_entry & PAGE_PRESENT) != 0 && (pdpt_entry & PAGE_USER) != 0 {
                    if (pdpt_entry & PAGE_HUGE) != 0 {
                        let frame = pdpt_entry & PTE_ADDR_MASK_1G;
                        if frame >= pmm::KERNEL_BASE_1MB
                            && (frame % 0x4000_0000) == 0
                            && pmm::is_valid_ram_range(frame, 0x4000_0000)
                        {
                            pmm::free_contiguous_frames(frame, 512 * 512);
                        }
                    } else {
                        let child_phys = pdpt_entry & PTE_ADDR_MASK;
                        if is_valid_page_table_frame(child_phys) {
                            free_user_page_table_subtree(child_phys, 2);
                        }
                    }
                }
            }

            if pdpt_phys >= pmm::KERNEL_BASE_1MB {
                pmm::free_frame(pdpt_phys);
            }
        }
    }

    for i in 1..256 {
        let entry = *pml4.add(i);
        if (entry & PAGE_PRESENT) != 0 && (entry & PAGE_USER) != 0 {
            let child_phys = entry & PTE_ADDR_MASK;
            if is_valid_page_table_frame(child_phys) {
                free_user_page_table_subtree(child_phys, 3);
            }
        }
    }

    cleanup_vmas_for_pml4(pml4_phys);

    if pml4_phys >= pmm::KERNEL_BASE_1MB {
        pmm::free_frame(pml4_phys);
    }
}

/// Recursively walks a user page table tree, reclaiming allocated page frames.
///
/// * `level == 3`: PDPT level.
/// * `level == 2`: PD level.
/// * `level == 1`: PT level.
///
/// # Safety
///
/// Deallocates physical memory and dereferences raw physical addresses.
unsafe fn free_user_page_table_subtree(table_phys: u64, level: u32) {
    if !is_valid_page_table_frame(table_phys) {
        return;
    }

    let table = table_phys as *const u64;

    if level == 1 {
        for i in 0..512 {
            let entry = *table.add(i);
            if (entry & PAGE_PRESENT) != 0 && (entry & PAGE_USER) != 0 {
                let frame = entry & PTE_ADDR_MASK;
                if frame >= pmm::KERNEL_BASE_1MB && pmm::is_valid_ram_range(frame, pmm::PAGE_SIZE) {
                    pmm::free_frame(frame);
                }
            }
        }
    } else {
        for i in 0..512 {
            let entry = *table.add(i);
            if (entry & PAGE_PRESENT) != 0 && (entry & PAGE_USER) != 0 {
                if (entry & PAGE_HUGE) != 0 {
                    let (frame, count, size) = if level == 3 {
                        (entry & PTE_ADDR_MASK_1G, 512 * 512, 0x4000_0000)
                    } else {
                        (entry & PTE_ADDR_MASK_2M, 512, 0x20_0000)
                    };
                    if frame >= pmm::KERNEL_BASE_1MB && pmm::is_valid_ram_range(frame, size) {
                        pmm::free_contiguous_frames(frame, count);
                    }
                } else {
                    let child_phys = entry & PTE_ADDR_MASK;
                    if is_valid_page_table_frame(child_phys) {
                        free_user_page_table_subtree(child_phys, level - 1);
                    }
                }
            }
        }
    }

    if table_phys >= pmm::KERNEL_BASE_1MB {
        pmm::free_frame(table_phys);
    }
}
