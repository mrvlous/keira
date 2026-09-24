// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Page table permission updates and user mapping validation.

use super::super::table::{
    active_pml4, PAGE_COW, PAGE_HUGE, PAGE_PRESENT, PAGE_USER, PAGE_WRITABLE, PTE_ADDR_MASK,
    PTE_ADDR_MASK_1G, PTE_ADDR_MASK_2M,
};
use crate::pmm;
use keira_arch::cpu::invlpg;

/// Modifies access flags for an active page and flushes the TLB.
///
/// # Safety
///
/// Directly modifies hardware page table permissions and invalidates the processor TLB.
pub unsafe fn mprotect_page(virtual_addr: u64, new_flags: u64) -> Result<(), &'static str> {
    if (virtual_addr % pmm::PAGE_SIZE) != 0 {
        return Err("Virtual address is not page-aligned");
    }

    let pml4_idx = ((virtual_addr >> 39) & 0x1FF) as usize;
    let pdpt_idx = ((virtual_addr >> 30) & 0x1FF) as usize;
    let pd_idx = ((virtual_addr >> 21) & 0x1FF) as usize;
    let pt_idx = ((virtual_addr >> 12) & 0x1FF) as usize;

    let pml4_addr = active_pml4();
    let pml4 = pml4_addr as *mut u64;

    let pml4_entry = *pml4.add(pml4_idx);
    if (pml4_entry & PAGE_PRESENT) == 0 {
        return Err("Page not mapped (PDPT missing)");
    }
    let pdpt = (pml4_entry & PTE_ADDR_MASK) as *mut u64;

    let pdpt_entry = *pdpt.add(pdpt_idx);
    if (pdpt_entry & PAGE_PRESENT) == 0 {
        return Err("Page not mapped (PD missing)");
    }
    if (pdpt_entry & PAGE_HUGE) != 0 {
        if (virtual_addr % 0x4000_0000) == 0 {
            let phys_frame = pdpt_entry & PTE_ADDR_MASK_1G;
            *pdpt.add(pdpt_idx) = phys_frame | new_flags | PAGE_PRESENT | PAGE_USER | PAGE_HUGE;
            invlpg(virtual_addr as usize);
            return Ok(());
        }
        return Err("Cannot mprotect sub-page of 1GB huge page without splitting");
    }
    let pd = (pdpt_entry & PTE_ADDR_MASK) as *mut u64;

    let pd_entry = *pd.add(pd_idx);
    if (pd_entry & PAGE_PRESENT) == 0 {
        return Err("Page not mapped (PT missing)");
    }
    if (pd_entry & PAGE_HUGE) != 0 {
        if (virtual_addr % 0x20_0000) == 0 {
            let phys_frame = pd_entry & PTE_ADDR_MASK_2M;
            *pd.add(pd_idx) = phys_frame | new_flags | PAGE_PRESENT | PAGE_USER | PAGE_HUGE;
            invlpg(virtual_addr as usize);
            return Ok(());
        }
        return Err("Cannot mprotect sub-page of 2MB huge page without splitting");
    }
    let pt = (pd_entry & PTE_ADDR_MASK) as *mut u64;

    let pt_entry = *pt.add(pt_idx);
    if (pt_entry & PAGE_PRESENT) == 0 {
        return Err("Page not mapped");
    }

    let phys_frame = pt_entry & PTE_ADDR_MASK;
    *pt.add(pt_idx) = phys_frame | new_flags | PAGE_PRESENT | PAGE_USER;

    invlpg(virtual_addr as usize);
    Ok(())
}

/// Checks whether a virtual address is mapped with user-mode access in the active address space.
///
/// If `require_writable` is set, verifies that write privileges are permitted (or guarded by COW).
///
/// # Safety
///
/// Inspects hardware page tables via CR3.
pub unsafe fn is_user_page_mapped(virtual_addr: u64, require_writable: bool) -> bool {
    let pml4_idx = ((virtual_addr >> 39) & 0x1FF) as usize;
    let pdpt_idx = ((virtual_addr >> 30) & 0x1FF) as usize;
    let pd_idx = ((virtual_addr >> 21) & 0x1FF) as usize;
    let pt_idx = ((virtual_addr >> 12) & 0x1FF) as usize;

    let pml4 = active_pml4() as *const u64;
    let pml4_entry = *pml4.add(pml4_idx);
    if (pml4_entry & PAGE_PRESENT) == 0 || (pml4_entry & PAGE_USER) == 0 {
        return false;
    }

    let pdpt = (pml4_entry & PTE_ADDR_MASK) as *const u64;
    let pdpt_entry = *pdpt.add(pdpt_idx);
    if (pdpt_entry & PAGE_PRESENT) == 0 || (pdpt_entry & PAGE_USER) == 0 {
        return false;
    }
    if (pdpt_entry & PAGE_HUGE) != 0 {
        if require_writable && (pdpt_entry & PAGE_WRITABLE) == 0 && (pdpt_entry & PAGE_COW) == 0 {
            return false;
        }
        return true;
    }

    let pd = (pdpt_entry & PTE_ADDR_MASK) as *const u64;
    let pd_entry = *pd.add(pd_idx);
    if (pd_entry & PAGE_PRESENT) == 0 || (pd_entry & PAGE_USER) == 0 {
        return false;
    }
    if (pd_entry & PAGE_HUGE) != 0 {
        if require_writable && (pd_entry & PAGE_WRITABLE) == 0 && (pd_entry & PAGE_COW) == 0 {
            return false;
        }
        return true;
    }

    let pt = (pd_entry & PTE_ADDR_MASK) as *const u64;
    let pt_entry = *pt.add(pt_idx);
    if (pt_entry & PAGE_PRESENT) == 0 || (pt_entry & PAGE_USER) == 0 {
        return false;
    }

    if require_writable && (pt_entry & PAGE_WRITABLE) == 0 && (pt_entry & PAGE_COW) == 0 {
        return false;
    }

    true
}
