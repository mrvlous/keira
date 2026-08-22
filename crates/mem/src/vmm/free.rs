// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Safe bounded user-space memory page and page table tree deallocation.

use super::paging::{
    active_pml4, free_and_unmap_page, switch_address_space, PAGE_PRESENT, PTE_ADDR_MASK,
};
use crate::pmm;

/// Free all user-space pages and page table frames from a process's PML4.
/// Must be called while a DIFFERENT address space is active (e.g., boot PML4).
pub unsafe fn free_user_pages(pml4_phys: u64, program_break: u64) {
    let saved_cr3 = active_pml4();
    switch_address_space(pml4_phys);

    // 1. Free user code/data pages (0x40000000 .. 0x40040000 = 256KB range)
    let mut addr: u64 = 0x40000000;
    while addr < 0x40040000 {
        let _ = free_and_unmap_page(addr);
        addr += pmm::PAGE_SIZE;
    }

    // 2. Free user heap pages (0x600000000000 .. program_break)
    addr = 0x600000000000;
    while addr < program_break {
        let _ = free_and_unmap_page(addr);
        addr += pmm::PAGE_SIZE;
    }

    // 3. Free user stack pages (0x7FFFFFE00000 .. 0x7FFFFFFF0000 = 64KB range)
    let mut stack_addr: u64 = 0x7FFFFFE00000;
    while stack_addr < 0x7FFFFFFF0000 {
        let _ = free_and_unmap_page(stack_addr);
        stack_addr += pmm::PAGE_SIZE;
    }

    switch_address_space(saved_cr3);

    // 4. Free the child's page table frames (PDPT and intermediate PD/PT tables)
    let pml4 = pml4_phys as *const u64;
    let pml4_0 = *pml4;
    if (pml4_0 & PAGE_PRESENT) != 0 {
        let pdpt_phys = pml4_0 & PTE_ADDR_MASK;
        let pdpt = pdpt_phys as *const u64;

        // Free user page table structures under PDPT[1..2] only (user code area under PML4[0]).
        // Do NOT free PDPT[0] (kernel identity map) or PDPT[3] (kernel MMIO/framebuffer).
        for i in 1..3 {
            let pdpt_entry = *pdpt.add(i);
            if (pdpt_entry & PAGE_PRESENT) != 0 {
                free_page_table_tree(pdpt_entry & PTE_ADDR_MASK, 2);
            }
        }
        pmm::free_frame(pdpt_phys);
    }

    // Free page table structures under PML4[1..511] (user heap/stack)
    for i in 1..512 {
        let entry = *pml4.add(i);
        if (entry & PAGE_PRESENT) != 0 {
            free_page_table_tree(entry & PTE_ADDR_MASK, 3);
        }
    }

    // Free the PML4 frame itself
    pmm::free_frame(pml4_phys);
}

/// Recursively free page table frames at a given level.
/// Level 3 = PDPT, Level 2 = PD, Level 1 = PT
unsafe fn free_page_table_tree(table_phys: u64, level: u32) {
    let table = table_phys as *const u64;
    if level > 1 {
        for i in 0..512 {
            let entry = *table.add(i);
            if (entry & PAGE_PRESENT) != 0 {
                // Skip huge pages (bit 7)
                if (entry & (1 << 7)) == 0 {
                    free_page_table_tree(entry & PTE_ADDR_MASK, level - 1);
                }
            }
        }
    }
    pmm::free_frame(table_phys);
}
