// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Page unmapping and frame reclamation routines for standard and huge pages.

use super::super::table::{
    active_pml4, get_pte_in_pml4, PAGE_HUGE, PAGE_PRESENT, PAGE_USER, PTE_ADDR_MASK,
    PTE_ADDR_MASK_1G, PTE_ADDR_MASK_2M,
};
use crate::pmm;
use keira_arch::cpu::invlpg;

/// Unmaps a virtual page from the active address space and invalidates the TLB.
///
/// # Safety
///
/// Clears page table entries in active hardware page tables and flushes the TLB.
pub unsafe fn unmap_page(virtual_addr: u64) -> Result<(), &'static str> {
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
            *pdpt.add(pdpt_idx) = 0;
            invlpg(virtual_addr as usize);
            return Ok(());
        }
        return Err("Cannot unmap sub-page of 1GB huge page without splitting");
    }

    let pd = (pdpt_entry & PTE_ADDR_MASK) as *mut u64;
    let pd_entry = *pd.add(pd_idx);
    if (pd_entry & PAGE_PRESENT) == 0 {
        return Err("Page not mapped (PT missing)");
    }
    if (pd_entry & PAGE_HUGE) != 0 {
        if (virtual_addr % 0x20_0000) == 0 {
            *pd.add(pd_idx) = 0;
            invlpg(virtual_addr as usize);
            return Ok(());
        }
        return Err("Cannot unmap sub-page of 2MB huge page without splitting");
    }

    let pt = (pd_entry & PTE_ADDR_MASK) as *mut u64;
    let pt_entry = *pt.add(pt_idx);
    if (pt_entry & PAGE_PRESENT) == 0 {
        return Err("Page not mapped");
    }

    *pt.add(pt_idx) = 0;
    invlpg(virtual_addr as usize);

    Ok(())
}

/// Unmaps a 2 MiB huge page from the Level 2 Page Directory.
///
/// # Safety
///
/// Clears huge page directory entries and flushes the TLB.
pub unsafe fn unmap_huge_2m_page(virtual_addr: u64) -> Result<(), &'static str> {
    if (virtual_addr % 0x20_0000) != 0 {
        return Err("Virtual address is not 2MB aligned");
    }

    let pml4_idx = ((virtual_addr >> 39) & 0x1FF) as usize;
    let pdpt_idx = ((virtual_addr >> 30) & 0x1FF) as usize;
    let pd_idx = ((virtual_addr >> 21) & 0x1FF) as usize;

    let pml4_phys = active_pml4();
    let pml4 = pml4_phys as *mut u64;

    let pml4_entry = *pml4.add(pml4_idx);
    if (pml4_entry & PAGE_PRESENT) == 0 {
        return Err("Page not mapped (PDPT missing)");
    }
    let pdpt = (pml4_entry & PTE_ADDR_MASK) as *mut u64;

    let pdpt_entry = *pdpt.add(pdpt_idx);
    if (pdpt_entry & PAGE_PRESENT) == 0 {
        return Err("Page not mapped (PD missing)");
    }
    let pd = (pdpt_entry & PTE_ADDR_MASK) as *mut u64;

    let pd_entry = *pd.add(pd_idx);
    if (pd_entry & PAGE_PRESENT) == 0 || (pd_entry & PAGE_HUGE) == 0 {
        return Err("2MB huge page not mapped");
    }

    *pd.add(pd_idx) = 0;
    invlpg(virtual_addr as usize);
    Ok(())
}

/// Unmaps a virtual page and returns its physical frame to the PMM allocator.
///
/// # Safety
///
/// Modifies active page tables and deallocates physical memory.
pub unsafe fn free_and_unmap_page(virtual_addr: u64) -> Result<(), &'static str> {
    if let Some(entry) = get_pte_in_pml4(active_pml4(), virtual_addr) {
        if (entry & PAGE_HUGE) != 0 {
            let is_1gb = (virtual_addr % 0x4000_0000) == 0;
            let frame = if is_1gb {
                entry & PTE_ADDR_MASK_1G
            } else {
                entry & PTE_ADDR_MASK_2M
            };
            unmap_page(virtual_addr)?;
            if (entry & PAGE_USER) != 0 && frame >= pmm::KERNEL_BASE_1MB {
                let frame_count = if is_1gb { 512 * 512 } else { 512 };
                pmm::free_contiguous_frames(frame, frame_count);
            }
            return Ok(());
        }
        let frame = entry & PTE_ADDR_MASK;
        unmap_page(virtual_addr)?;
        if frame >= pmm::KERNEL_BASE_1MB {
            pmm::free_frame(frame);
        }
        Ok(())
    } else {
        Err("Virtual address not mapped")
    }
}
