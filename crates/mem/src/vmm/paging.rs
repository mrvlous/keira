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
pub const PAGE_COW: u64 = 1 << 9; // Bit 9: Software Copy-On-Write flag
pub const PAGE_NO_EXECUTE: u64 = 1 << 63;
/// Page Size (PS) flag indicating a 2MB huge page (PD level) or 1GB huge page (PDPT level).
pub const PAGE_HUGE: u64 = 1 << 7;
pub const GB_1_IDENTITY_MAP: u64 = 0x4000_0000;

/// Canonical User Space Virtual Address Boundaries (x86_64 48-bit canonical addressing).
pub const USER_MIN_VADDR: u64 = 0x0000_0000_0001_0000; // 64 KiB (Null trap guard boundary)
pub const USER_MAX_VADDR: u64 = 0x0000_7FFF_FFFF_FFFF; // Upper limit of canonical 47-bit lower half

/// Canonical physical address mask for standard 4KB page table entries (bits 12..51).
pub const PTE_ADDR_MASK_4K: u64 = 0x000F_FFFF_FFFF_F000;
/// Canonical physical address mask for 2MB huge page directory entries (bits 21..51).
pub const PTE_ADDR_MASK_2M: u64 = 0x000F_FFFF_FFE0_0000;
/// Canonical physical address mask for 1GB huge page directory pointer entries (bits 30..51).
pub const PTE_ADDR_MASK_1G: u64 = 0x000F_FFFF_C000_0000;

/// Default canonical physical address mask alias for 4KB page tables.
pub const PTE_ADDR_MASK: u64 = PTE_ADDR_MASK_4K;

/// Resolve physical address from a page table entry and virtual address given the page level (3 = 1GiB, 2 = 2MiB, 1 = 4KiB).
pub fn translate_pte_to_phys(pte: u64, virtual_addr: u64, level: u8) -> u64 {
    match level {
        3 => (pte & PTE_ADDR_MASK_1G) | (virtual_addr & 0x3FFF_FFFF),
        2 => (pte & PTE_ADDR_MASK_2M) | (virtual_addr & 0x1F_FFFF),
        _ => (pte & PTE_ADDR_MASK_4K) | (virtual_addr & 0xFFF),
    }
}

pub static mut KASLR_SLIDE_OFFSET: u64 = 0x200000;

/// Return current KASLR slide offset.
pub fn get_kaslr_offset() -> u64 {
    unsafe { KASLR_SLIDE_OFFSET }
}

/// Get the physical base address of the active PML4 table from the CR3 register.
pub unsafe fn active_pml4() -> u64 {
    (read_cr3() as u64) & PTE_ADDR_MASK
}

/// Switch the active address space by writing a new PML4 physical address to CR3.
pub unsafe fn switch_address_space(pml4_phys: u64) {
    write_cr3(pml4_phys as usize);
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
        invlpg(virtual_addr as usize);
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
        if virtual_addr.is_multiple_of(0x4000_0000) {
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
        if virtual_addr.is_multiple_of(0x20_0000) {
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

/// Check if a user page is currently mapped in active PML4, optionally verifying write permission.
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
        if require_writable && (pdpt_entry & PAGE_WRITABLE) == 0 {
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
        if require_writable && (pd_entry & PAGE_WRITABLE) == 0 {
            return false;
        }
        return true;
    }

    let pt = (pd_entry & PTE_ADDR_MASK) as *const u64;
    let pt_entry = *pt.add(pt_idx);
    if (pt_entry & PAGE_PRESENT) == 0 || (pt_entry & PAGE_USER) == 0 {
        return false;
    }

    if require_writable && (pt_entry & PAGE_WRITABLE) == 0 {
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
        if virtual_addr.is_multiple_of(0x4000_0000) {
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
        if virtual_addr.is_multiple_of(0x20_0000) {
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

/// Unmap a virtual page and free its underlying physical frame.
pub unsafe fn free_and_unmap_page(virtual_addr: u64) -> Result<(), &'static str> {
    if let Some(entry) = get_pte_in_pml4(active_pml4(), virtual_addr) {
        if (entry & PAGE_HUGE) != 0 {
            let is_1gb = virtual_addr.is_multiple_of(0x4000_0000);
            let frame = if is_1gb {
                entry & PTE_ADDR_MASK_1G
            } else {
                entry & PTE_ADDR_MASK_2M
            };
            unmap_page(virtual_addr)?;
            if (entry & PAGE_USER) != 0 && frame >= pmm::KERNEL_BASE_1MB {
                let frame_count = if is_1gb {
                    512 * 512 // 1GB in 4KB frames
                } else {
                    512 // 2MB in 4KB frames
                };
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
        let pml4_entry = *pml4.add(pml4_idx);
        if (pml4_entry & PAGE_PRESENT) == 0 {
            return None;
        }

        let pdpt = (pml4_entry & PTE_ADDR_MASK) as *const u64;
        let pdpt_entry = *pdpt.add(pdpt_idx);
        if (pdpt_entry & PAGE_PRESENT) == 0 {
            return None;
        }
        if (pdpt_entry & PAGE_HUGE) != 0 {
            // 1GB huge page entry at PDPT level
            return Some(pdpt_entry);
        }

        let pd = (pdpt_entry & PTE_ADDR_MASK) as *const u64;
        let pd_entry = *pd.add(pd_idx);
        if (pd_entry & PAGE_PRESENT) == 0 {
            return None;
        }
        if (pd_entry & PAGE_HUGE) != 0 {
            // 2MB huge page entry at PD level
            return Some(pd_entry);
        }

        let pt = (pd_entry & PTE_ADDR_MASK) as *const u64;
        let pt_entry = *pt.add(pt_idx);
        if (pt_entry & PAGE_PRESENT) == 0 {
            return None;
        }

        Some(pt_entry)
    }
}

/// Retrieve a mutable pointer to the raw page table entry (PTE) for a virtual address in a specific PML4 table.
pub unsafe fn get_pte_mut_in_pml4(pml4_phys: u64, virtual_addr: u64) -> Option<*mut u64> {
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
        let pml4_entry = *pml4.add(pml4_idx);
        if (pml4_entry & PAGE_PRESENT) == 0 {
            return None;
        }

        let pdpt = (pml4_entry & PTE_ADDR_MASK) as *const u64;
        let pdpt_entry = *pdpt.add(pdpt_idx);
        if (pdpt_entry & PAGE_PRESENT) == 0 || (pdpt_entry & PAGE_HUGE) != 0 {
            return None;
        }

        let pd = (pdpt_entry & PTE_ADDR_MASK) as *const u64;
        let pd_entry = *pd.add(pd_idx);
        if (pd_entry & PAGE_PRESENT) == 0 || (pd_entry & PAGE_HUGE) != 0 {
            return None;
        }

        let pt = (pd_entry & PTE_ADDR_MASK) as *mut u64;
        let pt_entry = *pt.add(pt_idx);
        if (pt_entry & PAGE_PRESENT) == 0 {
            return None;
        }

        Some(pt.add(pt_idx))
    }
}

/// Check if a virtual address is present in the specified PML4 table.
pub unsafe fn is_page_mapped_in_pml4(pml4_phys: u64, virtual_addr: u64) -> bool {
    get_pte_in_pml4(pml4_phys, virtual_addr).is_some()
}

/// Translate a virtual address to its corresponding physical address within a specific PML4 table.
pub unsafe fn get_phys_addr_in_pml4(pml4_phys: u64, virtual_addr: u64) -> Option<u64> {
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
        let pml4_entry = *pml4.add(pml4_idx);
        if (pml4_entry & PAGE_PRESENT) == 0 {
            return None;
        }

        let pdpt = (pml4_entry & PTE_ADDR_MASK) as *const u64;
        let pdpt_entry = *pdpt.add(pdpt_idx);
        if (pdpt_entry & PAGE_PRESENT) == 0 {
            return None;
        }
        if (pdpt_entry & PAGE_HUGE) != 0 {
            return Some(translate_pte_to_phys(pdpt_entry, virtual_addr, 3));
        }

        let pd = (pdpt_entry & PTE_ADDR_MASK) as *const u64;
        let pd_entry = *pd.add(pd_idx);
        if (pd_entry & PAGE_PRESENT) == 0 {
            return None;
        }
        if (pd_entry & PAGE_HUGE) != 0 {
            return Some(translate_pte_to_phys(pd_entry, virtual_addr, 2));
        }

        let pt = (pd_entry & PTE_ADDR_MASK) as *const u64;
        let pt_entry = *pt.add(pt_idx);
        if (pt_entry & PAGE_PRESENT) == 0 {
            return None;
        }

        Some(translate_pte_to_phys(pt_entry, virtual_addr, 1))
    }
}

/// Translate a virtual address to its corresponding physical address, stripping all PTE flag and NX bits.
pub unsafe fn get_phys_addr(virtual_addr: u64) -> Option<u64> {
    get_phys_addr_in_pml4(active_pml4(), virtual_addr)
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

    #[test]
    fn test_translate_4k_page() {
        let frame: u64 = 0x0000_0000_1234_5000;
        let pte = frame | PAGE_PRESENT | PAGE_WRITABLE | PAGE_USER | PAGE_NO_EXECUTE;
        let vaddr: u64 = 0x5000_0000_1ABC;
        let phys = translate_pte_to_phys(pte, vaddr, 1);
        assert_eq!(phys, 0x0000_0000_1234_5ABC);
    }

    #[test]
    fn test_translate_2mib_huge_page() {
        let frame: u64 = 0x0000_0000_2000_0000; // 2MB aligned
        let pde = frame | PAGE_PRESENT | PAGE_WRITABLE | PAGE_USER | PAGE_HUGE;
        let vaddr: u64 = 0x5000_0012_3456;
        let phys = translate_pte_to_phys(pde, vaddr, 2);
        // Offset within 2MB: 0x12_3456. Result: 0x2012_3456
        assert_eq!(phys, 0x0000_0000_2012_3456);
    }

    #[test]
    fn test_translate_1gib_huge_page() {
        let frame: u64 = 0x0000_0000_8000_0000; // 1GB aligned
        let pdpte = frame | PAGE_PRESENT | PAGE_WRITABLE | PAGE_USER | PAGE_HUGE;
        let vaddr: u64 = 0x5000_1234_5678;
        let phys = translate_pte_to_phys(pdpte, vaddr, 3);
        // Offset within 1GB: 0x1234_5678. Result: 0x9234_5678
        assert_eq!(phys, 0x0000_0000_9234_5678);
    }

    #[test]
    fn test_huge_page_masks_and_alignment() {
        let frame_1gb = 0x4000_0000u64;
        let pte_1gb = frame_1gb | PAGE_PRESENT | PAGE_WRITABLE | PAGE_USER | PAGE_HUGE;
        assert_eq!(pte_1gb & PTE_ADDR_MASK_1G, frame_1gb);

        let frame_2mb = 0x20_0000u64;
        let pte_2mb = frame_2mb | PAGE_PRESENT | PAGE_WRITABLE | PAGE_USER | PAGE_HUGE;
        assert_eq!(pte_2mb & PTE_ADDR_MASK_2M, frame_2mb);

        let frame_4k = 0x1000u64;
        let pte_4k = frame_4k | PAGE_PRESENT | PAGE_WRITABLE | PAGE_USER;
        assert_eq!(pte_4k & PTE_ADDR_MASK_4K, frame_4k);
    }
}
