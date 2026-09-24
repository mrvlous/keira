// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! PML4 address space cloning preserving kernel identity mapping, MMIO, and process isolation.

use super::super::mapping::map_page_in_pml4;
use super::super::reclaim::free_user_pages;
use super::super::table::{
    active_pml4, switch_address_space, PAGE_HUGE, PAGE_NO_EXECUTE, PAGE_PRESENT, PAGE_USER,
    PAGE_WRITABLE, PTE_ADDR_MASK,
};
use crate::pmm;

/// Clones the boot PML4 table, sharing the kernel identity mapping (PDPT[0]) and MMIO (PDPT[3]).
///
/// User-space address ranges are left unpopulated for the child process.
///
/// # Safety
///
/// Directly reads physical memory, modifies page tables, and allocates page frames.
pub unsafe fn clone_kernel_pml4() -> Result<u64, &'static str> {
    let boot_pml4_phys = active_pml4();
    let boot_pml4 = boot_pml4_phys as *const u64;

    let new_pml4_phys = pmm::alloc_frame().ok_or("Out of memory for new PML4")?;
    let new_pml4 = new_pml4_phys as *mut u64;

    let boot_pml4_0 = *boot_pml4;
    if (boot_pml4_0 & PAGE_PRESENT) == 0 {
        pmm::free_frame(new_pml4_phys);
        return Err("Boot PML4[0] is not present");
    }

    let boot_pdpt_phys = boot_pml4_0 & PTE_ADDR_MASK;
    let boot_pdpt = boot_pdpt_phys as *const u64;

    let new_pdpt_phys = match pmm::alloc_frame() {
        Some(p) => p,
        None => {
            pmm::free_frame(new_pml4_phys);
            return Err("Out of memory for new PDPT");
        }
    };
    let new_pdpt = new_pdpt_phys as *mut u64;

    *new_pdpt.add(0) = (*boot_pdpt.add(0)) & !PAGE_USER;
    *new_pdpt.add(3) = (*boot_pdpt.add(3)) & !PAGE_USER;

    *new_pml4 = (new_pdpt_phys & PTE_ADDR_MASK) | PAGE_PRESENT | PAGE_WRITABLE | PAGE_USER;

    for i in 256..512 {
        let entry = *boot_pml4.add(i);
        if (entry & PAGE_PRESENT) != 0 {
            *new_pml4.add(i) = entry;
        }
    }

    Ok(new_pml4_phys)
}

/// Clones an entire parent process address space into a newly allocated PML4 hierarchy.
///
/// Performs deep page duplication for all user-mapped frames to enforce memory isolation.
///
/// # Safety
///
/// Modifies address space configurations, page tables, and copies frame contents.
pub unsafe fn clone_user_address_space(parent_pml4_phys: u64) -> Result<u64, &'static str> {
    let child_pml4_phys = clone_kernel_pml4()?;

    let current_pml4 = active_pml4();
    switch_address_space(parent_pml4_phys);

    let parent_pml4 = parent_pml4_phys as *const u64;

    for pml4_idx in 0..256 {
        let pml4_entry = *parent_pml4.add(pml4_idx);
        if (pml4_entry & PAGE_PRESENT) == 0 {
            continue;
        }

        if pml4_idx > 0 && (pml4_entry & PAGE_USER) == 0 {
            continue;
        }

        let pdpt_phys = pml4_entry & PTE_ADDR_MASK;
        if !pmm::is_valid_ram_range(pdpt_phys, pmm::PAGE_SIZE) {
            continue;
        }
        let pdpt = pdpt_phys as *const u64;

        for pdpt_idx in 0..512 {
            if pml4_idx == 0 && (pdpt_idx == 0 || pdpt_idx == 3) {
                continue;
            }

            let pdpt_entry = *pdpt.add(pdpt_idx);
            if (pdpt_entry & PAGE_PRESENT) == 0 || (pdpt_entry & PAGE_USER) == 0 {
                continue;
            }

            if (pdpt_entry & PAGE_HUGE) != 0 {
                continue;
            }

            let pd_phys = pdpt_entry & PTE_ADDR_MASK;
            if !pmm::is_valid_ram_range(pd_phys, pmm::PAGE_SIZE) {
                continue;
            }
            let pd = pd_phys as *const u64;

            for pd_idx in 0..512 {
                let pd_entry = *pd.add(pd_idx);
                if (pd_entry & PAGE_PRESENT) == 0 || (pd_entry & PAGE_USER) == 0 {
                    continue;
                }

                if (pd_entry & PAGE_HUGE) != 0 {
                    continue;
                }

                let pt_phys = pd_entry & PTE_ADDR_MASK;
                if !pmm::is_valid_ram_range(pt_phys, pmm::PAGE_SIZE) {
                    continue;
                }
                let pt = pt_phys as *const u64;

                for pt_idx in 0..512 {
                    let pt_entry = *pt.add(pt_idx);
                    if (pt_entry & PAGE_PRESENT) == 0 || (pt_entry & PAGE_USER) == 0 {
                        continue;
                    }

                    let mut vaddr = 0u64;
                    vaddr |= (pml4_idx as u64) << 39;
                    vaddr |= (pdpt_idx as u64) << 30;
                    vaddr |= (pd_idx as u64) << 21;
                    vaddr |= (pt_idx as u64) << 12;

                    let phys_frame = pt_entry & PTE_ADDR_MASK;
                    if !pmm::is_valid_ram_range(phys_frame, pmm::PAGE_SIZE) {
                        continue;
                    }

                    let child_frame = match pmm::alloc_frame() {
                        Some(f) => f,
                        None => {
                            switch_address_space(current_pml4);
                            free_user_pages(child_pml4_phys, 0x0000_7FFF_FFFF_FFFF);
                            pmm::free_frame(child_pml4_phys);
                            return Err("Out of memory for child user page frame");
                        }
                    };

                    core::ptr::copy_nonoverlapping(
                        phys_frame as *const u8,
                        child_frame as *mut u8,
                        4096,
                    );

                    if let Err(e) = map_page_in_pml4(
                        child_pml4_phys,
                        vaddr,
                        child_frame,
                        pt_entry & (PAGE_USER | PAGE_WRITABLE | PAGE_NO_EXECUTE | PAGE_PRESENT),
                    ) {
                        pmm::free_frame(child_frame);
                        switch_address_space(current_pml4);
                        free_user_pages(child_pml4_phys, 0x0000_7FFF_FFFF_FFFF);
                        pmm::free_frame(child_pml4_phys);
                        return Err(e);
                    }
                }
            }
        }
    }

    switch_address_space(current_pml4);
    Ok(child_pml4_phys)
}
