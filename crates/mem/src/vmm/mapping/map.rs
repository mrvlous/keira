// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Page mapping routines for standard 4 KiB frames and 2 MiB huge pages.

use super::super::table::{
    active_pml4, PAGE_HUGE, PAGE_PRESENT, PAGE_USER, PAGE_WRITABLE, PTE_ADDR_MASK, PTE_ADDR_MASK_2M,
};
use crate::pmm;
use keira_arch::cpu::invlpg;

/// Maps a virtual page to a physical frame within the active address space.
///
/// # Safety
///
/// Directly modifies hardware page table structures and invalidates the processor TLB.
pub unsafe fn map_page(
    virtual_addr: u64,
    physical_addr: u64,
    flags: u64,
) -> Result<(), &'static str> {
    map_page_in_pml4(active_pml4(), virtual_addr, physical_addr, flags)
}

/// Maps a virtual page to a physical frame within a designated PML4 table.
///
/// # Safety
///
/// Traverses and populates page table frames (PDPT, PD, PT) via physical addresses.
pub unsafe fn map_page_in_pml4(
    pml4_phys: u64,
    virtual_addr: u64,
    physical_addr: u64,
    flags: u64,
) -> Result<(), &'static str> {
    if (virtual_addr % pmm::PAGE_SIZE) != 0 || (physical_addr % pmm::PAGE_SIZE) != 0 {
        return Err("Virtual or Physical address is not page-aligned");
    }

    let pml4_idx = ((virtual_addr >> 39) & 0x1FF) as usize;
    let pdpt_idx = ((virtual_addr >> 30) & 0x1FF) as usize;
    let pd_idx = ((virtual_addr >> 21) & 0x1FF) as usize;
    let pt_idx = ((virtual_addr >> 12) & 0x1FF) as usize;

    let pml4 = pml4_phys as *mut u64;

    let pdpt_entry = *pml4.add(pml4_idx);
    let mut newly_allocated_pdpt: Option<u64> = None;
    let pdpt_addr = if (pdpt_entry & PAGE_PRESENT) == 0 {
        let frame = pmm::alloc_frame().ok_or("Out of physical memory for PDPT")?;
        *pml4.add(pml4_idx) = frame | PAGE_PRESENT | PAGE_WRITABLE | PAGE_USER;
        newly_allocated_pdpt = Some(frame);
        frame
    } else {
        if (flags & PAGE_USER) != 0 {
            *pml4.add(pml4_idx) |= PAGE_USER;
        }
        pdpt_entry & PTE_ADDR_MASK
    };
    let pdpt = pdpt_addr as *mut u64;

    let pd_entry = *pdpt.add(pdpt_idx);
    let mut newly_allocated_pd: Option<u64> = None;
    let pd_addr = if (pd_entry & PAGE_PRESENT) == 0 {
        let frame = match pmm::alloc_frame() {
            Some(f) => f,
            None => {
                if let Some(pdpt_frame) = newly_allocated_pdpt {
                    *pml4.add(pml4_idx) = 0;
                    pmm::free_frame(pdpt_frame);
                }
                return Err("Out of physical memory for PD");
            }
        };
        *pdpt.add(pdpt_idx) = frame | PAGE_PRESENT | PAGE_WRITABLE | PAGE_USER;
        newly_allocated_pd = Some(frame);
        frame
    } else {
        if (flags & PAGE_USER) != 0 {
            *pdpt.add(pdpt_idx) |= PAGE_USER;
        }
        pd_entry & PTE_ADDR_MASK
    };
    let pd = pd_addr as *mut u64;

    let pt_entry = *pd.add(pd_idx);
    let pt_addr = if (pt_entry & PAGE_PRESENT) == 0 {
        let frame = match pmm::alloc_frame() {
            Some(f) => f,
            None => {
                if let Some(pd_frame) = newly_allocated_pd {
                    *pdpt.add(pdpt_idx) = 0;
                    pmm::free_frame(pd_frame);
                }
                if let Some(pdpt_frame) = newly_allocated_pdpt {
                    *pml4.add(pml4_idx) = 0;
                    pmm::free_frame(pdpt_frame);
                }
                return Err("Out of physical memory for PT");
            }
        };
        *pd.add(pd_idx) = frame | PAGE_PRESENT | PAGE_WRITABLE | PAGE_USER;
        frame
    } else {
        if (flags & PAGE_USER) != 0 {
            *pd.add(pd_idx) |= PAGE_USER;
        }
        pt_entry & PTE_ADDR_MASK
    };
    let pt = pt_addr as *mut u64;

    *pt.add(pt_idx) = (physical_addr & PTE_ADDR_MASK) | flags | PAGE_PRESENT;

    if pml4_phys == active_pml4() {
        invlpg(virtual_addr as usize);
    }

    Ok(())
}

/// Maps a 2 MiB huge page directly into the Level 2 Page Directory.
///
/// # Safety
///
/// Modifies hardware page directories and flushes the TLB.
pub unsafe fn map_huge_2m_page(
    virtual_addr: u64,
    physical_addr: u64,
    flags: u64,
) -> Result<(), &'static str> {
    if (virtual_addr % 0x20_0000) != 0 {
        return Err("Virtual address is not 2MB aligned");
    }
    if (physical_addr % 0x20_0000) != 0 {
        return Err("Physical address is not 2MB aligned");
    }

    let pml4_idx = ((virtual_addr >> 39) & 0x1FF) as usize;
    let pdpt_idx = ((virtual_addr >> 30) & 0x1FF) as usize;
    let pd_idx = ((virtual_addr >> 21) & 0x1FF) as usize;

    let pml4_phys = active_pml4();
    let pml4 = pml4_phys as *mut u64;

    let pml4_entry = *pml4.add(pml4_idx);
    let pdpt_addr = if (pml4_entry & PAGE_PRESENT) == 0 {
        let frame = pmm::alloc_frame().ok_or("Out of physical memory for PDPT")?;
        *pml4.add(pml4_idx) = frame | PAGE_PRESENT | PAGE_WRITABLE | PAGE_USER;
        frame
    } else {
        pml4_entry & PTE_ADDR_MASK
    };
    let pdpt = pdpt_addr as *mut u64;

    let pdpt_entry = *pdpt.add(pdpt_idx);
    let pd_addr = if (pdpt_entry & PAGE_PRESENT) == 0 {
        let frame = pmm::alloc_frame().ok_or("Out of physical memory for PD")?;
        *pdpt.add(pdpt_idx) = frame | PAGE_PRESENT | PAGE_WRITABLE | PAGE_USER;
        frame
    } else {
        pdpt_entry & PTE_ADDR_MASK
    };
    let pd = pd_addr as *mut u64;

    *pd.add(pd_idx) = (physical_addr & PTE_ADDR_MASK_2M) | flags | PAGE_PRESENT | PAGE_HUGE;

    invlpg(virtual_addr as usize);
    Ok(())
}
