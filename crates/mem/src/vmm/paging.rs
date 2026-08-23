// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! 4-Level x86_64 Page Table traversal, mapping, translation, protection, and TLB invalidation.

use crate::pmm;
use keira_arch::cpu::{invlpg, read_cr3, write_cr3};

pub const PAGE_PRESENT: u64 = 1 << 0;
pub const PAGE_WRITABLE: u64 = 1 << 1;
pub const PAGE_USER: u64 = 1 << 2;
pub const PAGE_NO_EXECUTE: u64 = 1 << 63;
/// Page Size (PS) flag indicating a 2MB huge page (PD level) or 1GB huge page (PDPT level).
pub const PAGE_HUGE: u64 = 1 << 7;
pub const GB_1_IDENTITY_MAP: u64 = 0x4000_0000;

/// Canonical User Space Virtual Address Boundaries (x86_64 48-bit canonical addressing).
pub const USER_MIN_VADDR: u64 = 0x0000_0000_0001_0000; // 64 KiB (Null trap guard boundary)
pub const USER_MAX_VADDR: u64 = 0x0000_7FFF_FFFF_FFFF; // Upper limit of canonical 47-bit lower half

/// Canonical x86_64 physical address mask for page table entries (bits 12..51).
/// Strips lower 12-bit flags (Present, Writable, User, Huge) and upper bits including PAGE_NO_EXECUTE (bit 63).
pub const PTE_ADDR_MASK: u64 = 0x000F_FFFF_FFFF_F000;

pub static mut KASLR_SLIDE_OFFSET: u64 = 0x200000;

/// Return current KASLR slide offset.
pub fn get_kaslr_offset() -> u64 {
    unsafe { KASLR_SLIDE_OFFSET }
}

/// Get the physical base address of the active PML4 table from the CR3 register.
pub unsafe fn active_pml4() -> u64 {
    read_cr3() & PTE_ADDR_MASK
}

/// Switch the active address space by writing a new PML4 physical address to CR3.
pub unsafe fn switch_address_space(pml4_phys: u64) {
    write_cr3(pml4_phys);
}

/// Map a virtual page to a physical frame in the active PML4 table.
pub unsafe fn map_page(
    virtual_addr: u64,
    physical_addr: u64,
    flags: u64,
) -> Result<(), &'static str> {
    map_page_in_pml4(active_pml4(), virtual_addr, physical_addr, flags)
}

/// Map a virtual page to a physical frame in a specific PML4 table.
pub unsafe fn map_page_in_pml4(
    pml4_phys: u64,
    virtual_addr: u64,
    physical_addr: u64,
    flags: u64,
) -> Result<(), &'static str> {
    if !virtual_addr.is_multiple_of(pmm::PAGE_SIZE) || !physical_addr.is_multiple_of(pmm::PAGE_SIZE)
    {
        return Err("Virtual or Physical address is not page-aligned");
    }

    let pml4_idx = ((virtual_addr >> 39) & 0x1FF) as usize;
    let pdpt_idx = ((virtual_addr >> 30) & 0x1FF) as usize;
    let pd_idx = ((virtual_addr >> 21) & 0x1FF) as usize;
    let pt_idx = ((virtual_addr >> 12) & 0x1FF) as usize;

    let pml4 = pml4_phys as *mut u64;

    // 1. Traverse / Allocate PDPT
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

    // 2. Traverse / Allocate PD
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

    // 3. Traverse / Allocate PT
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

    // 4. Set PT entry
    *pt.add(pt_idx) = (physical_addr & PTE_ADDR_MASK) | flags | PAGE_PRESENT;

    // 5. Invalidate page in TLB if modifying active PML4
    if pml4_phys == active_pml4() {
        invlpg(virtual_addr);
    }

    Ok(())
}

/// Modify access permissions on an existing virtual page and invalidate TLB.
pub unsafe fn mprotect_page(virtual_addr: u64, new_flags: u64) -> Result<(), &'static str> {
    if !virtual_addr.is_multiple_of(pmm::PAGE_SIZE) {
        return Err("Virtual address is not page-aligned");
    }

    let pml4_idx = ((virtual_addr >> 39) & 0x1FF) as usize;
    let pdpt_idx = ((virtual_addr >> 30) & 0x1FF) as usize;
    let pd_idx = ((virtual_addr >> 21) & 0x1FF) as usize;
    let pt_idx = ((virtual_addr >> 12) & 0x1FF) as usize;

    let pml4_addr = active_pml4();
    let pml4 = pml4_addr as *mut u64;

    let pdpt_entry = *pml4.add(pml4_idx);
    if (pdpt_entry & PAGE_PRESENT) == 0 {
        return Err("Page not mapped (PDPT missing)");
    }
    let pdpt = (pdpt_entry & PTE_ADDR_MASK) as *mut u64;

    let pd_entry = *pdpt.add(pdpt_idx);
    if (pd_entry & PAGE_PRESENT) == 0 {
        return Err("Page not mapped (PD missing)");
    }
    let pd = (pd_entry & PTE_ADDR_MASK) as *mut u64;

    let pt_entry = *pd.add(pd_idx);
    if (pt_entry & PAGE_PRESENT) == 0 {
        return Err("Page not mapped (PT missing)");
    }
    let pt = (pt_entry & PTE_ADDR_MASK) as *mut u64;

    let entry = *pt.add(pt_idx);
    if (entry & PAGE_PRESENT) == 0 {
        return Err("Page not mapped");
    }

    let phys_frame = entry & PTE_ADDR_MASK;
    *pt.add(pt_idx) = phys_frame | new_flags | PAGE_PRESENT | PAGE_USER;

    invlpg(virtual_addr);
    Ok(())
}

/// Check if a user page is currently mapped in active PML4, optionally verifying write permission.
pub unsafe fn is_user_page_mapped(virtual_addr: u64, require_writable: bool) -> bool {
    let pml4_idx = ((virtual_addr >> 39) & 0x1FF) as usize;
    let pdpt_idx = ((virtual_addr >> 30) & 0x1FF) as usize;
    let pd_idx = ((virtual_addr >> 21) & 0x1FF) as usize;
    let pt_idx = ((virtual_addr >> 12) & 0x1FF) as usize;

    let pml4 = active_pml4() as *const u64;
    let pdpt_entry = *pml4.add(pml4_idx);
    if (pdpt_entry & PAGE_PRESENT) == 0 || (pdpt_entry & PAGE_USER) == 0 {
        return false;
    }

    let pdpt = (pdpt_entry & PTE_ADDR_MASK) as *const u64;
    let pd_entry = *pdpt.add(pdpt_idx);
    if (pd_entry & PAGE_PRESENT) == 0 || (pd_entry & PAGE_USER) == 0 {
        return false;
    }

    let pd = (pd_entry & PTE_ADDR_MASK) as *const u64;
    let pt_entry = *pd.add(pd_idx);
    if (pt_entry & PAGE_PRESENT) == 0 || (pt_entry & PAGE_USER) == 0 {
        return false;
    }

    let pt = (pt_entry & PTE_ADDR_MASK) as *const u64;
    let entry = *pt.add(pt_idx);
    if (entry & PAGE_PRESENT) == 0 || (entry & PAGE_USER) == 0 {
        return false;
    }

    if require_writable && (entry & PAGE_WRITABLE) == 0 {
        return false;
    }

    true
}

/// Unmap a virtual page.
pub unsafe fn unmap_page(virtual_addr: u64) -> Result<(), &'static str> {
    if !virtual_addr.is_multiple_of(pmm::PAGE_SIZE) {
        return Err("Virtual address is not page-aligned");
    }

    let pml4_idx = ((virtual_addr >> 39) & 0x1FF) as usize;
    let pdpt_idx = ((virtual_addr >> 30) & 0x1FF) as usize;
    let pd_idx = ((virtual_addr >> 21) & 0x1FF) as usize;
    let pt_idx = ((virtual_addr >> 12) & 0x1FF) as usize;

    let pml4_addr = active_pml4();
    let pml4 = pml4_addr as *mut u64;

    let pdpt_entry = *pml4.add(pml4_idx);
    if (pdpt_entry & PAGE_PRESENT) == 0 {
        return Err("Page not mapped (PDPT missing)");
    }
    let pdpt = (pdpt_entry & PTE_ADDR_MASK) as *mut u64;

    let pd_entry = *pdpt.add(pdpt_idx);
    if (pd_entry & PAGE_PRESENT) == 0 {
        return Err("Page not mapped (PD missing)");
    }
    let pd = (pd_entry & PTE_ADDR_MASK) as *mut u64;

    let pt_entry = *pd.add(pd_idx);
    if (pt_entry & PAGE_PRESENT) == 0 {
        return Err("Page not mapped (PT missing)");
    }
    let pt = (pt_entry & PTE_ADDR_MASK) as *mut u64;

    let entry = *pt.add(pt_idx);
    if (entry & PAGE_PRESENT) == 0 {
        return Err("Page not mapped");
    }

    *pt.add(pt_idx) = 0;
    invlpg(virtual_addr);

    Ok(())
}

/// Unmap a virtual page and free its underlying physical frame.
pub unsafe fn free_and_unmap_page(virtual_addr: u64) -> Result<(), &'static str> {
    if let Some(phys) = get_phys_addr(virtual_addr) {
        let frame = phys & PTE_ADDR_MASK;
        unmap_page(virtual_addr)?;
        pmm::free_frame(frame);
        Ok(())
    } else {
        Err("Virtual address not mapped")
    }
}

/// Retrieve the raw page table entry (PTE) for a virtual address in a specific PML4 table.
pub unsafe fn get_pte_in_pml4(pml4_phys: u64, virtual_addr: u64) -> Option<u64> {
    #[cfg(test)]
    {
        let _ = (pml4_phys, virtual_addr);
        return None;
    }
    #[cfg(not(test))]
    {
        if pml4_phys == 0 {
            return None;
        }
        let pml4_idx = ((virtual_addr >> 39) & 0x1FF) as usize;
        let pdpt_idx = ((virtual_addr >> 30) & 0x1FF) as usize;
        let pd_idx = ((virtual_addr >> 21) & 0x1FF) as usize;
        let pt_idx = ((virtual_addr >> 12) & 0x1FF) as usize;

        let pml4 = pml4_phys as *const u64;
        let pdpt_entry = *pml4.add(pml4_idx);
        if (pdpt_entry & PAGE_PRESENT) == 0 {
            return None;
        }

        let pdpt = (pdpt_entry & PTE_ADDR_MASK) as *const u64;
        let pd_entry = *pdpt.add(pdpt_idx);
        if (pd_entry & PAGE_PRESENT) == 0 {
            return None;
        }
        if (pd_entry & PAGE_HUGE) != 0 {
            // 2MB huge page entry
            return Some(pd_entry);
        }

        let pd = (pd_entry & PTE_ADDR_MASK) as *const u64;
        let pt_entry = *pd.add(pd_idx);
        if (pt_entry & PAGE_PRESENT) == 0 {
            return None;
        }

        let pt = (pt_entry & PTE_ADDR_MASK) as *const u64;
        let entry = *pt.add(pt_idx);
        if (entry & PAGE_PRESENT) == 0 {
            return None;
        }

        Some(entry)
    }
}

/// Check if a virtual address is present in the specified PML4 table.
pub unsafe fn is_page_mapped_in_pml4(pml4_phys: u64, virtual_addr: u64) -> bool {
    get_pte_in_pml4(pml4_phys, virtual_addr).is_some()
}

/// Translate a virtual address to its corresponding physical address, stripping all PTE flag and NX bits.
pub unsafe fn get_phys_addr(virtual_addr: u64) -> Option<u64> {
    let entry = get_pte_in_pml4(active_pml4(), virtual_addr)?;
    Some((entry & PTE_ADDR_MASK) | (virtual_addr & 0xFFF))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pte_addr_mask_strips_nx_and_flags() {
        let frame: u64 = 0x0000_0000_1234_5000;
        let pte_rwx = frame | PAGE_PRESENT | PAGE_WRITABLE | PAGE_USER;
        let pte_nx = frame | PAGE_PRESENT | PAGE_WRITABLE | PAGE_USER | PAGE_NO_EXECUTE;
        let pte_ro_nx = frame | PAGE_PRESENT | PAGE_USER | PAGE_NO_EXECUTE;

        assert_eq!(pte_rwx & PTE_ADDR_MASK, frame);
        assert_eq!(pte_nx & PTE_ADDR_MASK, frame);
        assert_eq!(pte_ro_nx & PTE_ADDR_MASK, frame);
        assert_eq!(
            (0x8000_0000_1234_5007u64 & PTE_ADDR_MASK),
            0x0000_0000_1234_5000u64
        );
    }
}
