// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! 4-Level page table traversal, lookup routines, and CR3 address space switching.

use super::entry::PTE_ADDR_MASK;
#[cfg(not(test))]
use super::entry::{translate_pte_to_phys, PAGE_HUGE, PAGE_PRESENT};
#[cfg(not(test))]
use crate::pmm;
use keira_arch::cpu::{read_cr3, write_cr3};

/// Retrieves the physical base address of the active PML4 table from CR3.
///
/// # Safety
///
/// Directly reads processor control register CR3.
pub unsafe fn active_pml4() -> u64 {
    (read_cr3() as u64) & PTE_ADDR_MASK
}

/// Switches the active address space by writing a new PML4 root physical address into CR3.
///
/// # Safety
///
/// Overwriting CR3 invalidates non-global TLB entries and switches memory translation root.
/// The caller must verify `pml4_phys` references a valid PML4 hierarchy.
pub unsafe fn switch_address_space(pml4_phys: u64) {
    write_cr3(pml4_phys as usize);
}

/// Retrieves the raw page table entry (PTE) value for a virtual address in a designated PML4 table.
///
/// # Safety
///
/// Dereferences page table pointers based on `pml4_phys`.
pub unsafe fn get_pte_in_pml4(pml4_phys: u64, virtual_addr: u64) -> Option<u64> {
    #[cfg(test)]
    {
        let _ = (pml4_phys, virtual_addr);
        return None;
    }
    #[cfg(not(test))]
    {
        if pml4_phys == 0 || !pmm::is_valid_ram_range(pml4_phys, pmm::PAGE_SIZE) {
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

        let pdpt_phys = pml4_entry & PTE_ADDR_MASK;
        if !pmm::is_valid_ram_range(pdpt_phys, pmm::PAGE_SIZE) {
            return None;
        }
        let pdpt = pdpt_phys as *const u64;
        let pdpt_entry = *pdpt.add(pdpt_idx);
        if (pdpt_entry & PAGE_PRESENT) == 0 {
            return None;
        }
        if (pdpt_entry & PAGE_HUGE) != 0 {
            return Some(pdpt_entry);
        }

        let pd_phys = pdpt_entry & PTE_ADDR_MASK;
        if !pmm::is_valid_ram_range(pd_phys, pmm::PAGE_SIZE) {
            return None;
        }
        let pd = pd_phys as *const u64;
        let pd_entry = *pd.add(pd_idx);
        if (pd_entry & PAGE_PRESENT) == 0 {
            return None;
        }
        if (pd_entry & PAGE_HUGE) != 0 {
            return Some(pd_entry);
        }

        let pt_phys = pd_entry & PTE_ADDR_MASK;
        if !pmm::is_valid_ram_range(pt_phys, pmm::PAGE_SIZE) {
            return None;
        }
        let pt = pt_phys as *const u64;
        let pt_entry = *pt.add(pt_idx);
        if (pt_entry & PAGE_PRESENT) == 0 {
            return None;
        }

        Some(pt_entry)
    }
}

/// Retrieves a mutable pointer to the raw page table entry (PTE) in a designated PML4 table.
///
/// # Safety
///
/// Dereferences physical memory pointers and returns a raw mutable pointer to the PTE.
pub unsafe fn get_pte_mut_in_pml4(pml4_phys: u64, virtual_addr: u64) -> Option<*mut u64> {
    #[cfg(test)]
    {
        let _ = (pml4_phys, virtual_addr);
        return None;
    }
    #[cfg(not(test))]
    {
        if pml4_phys == 0 || !pmm::is_valid_ram_range(pml4_phys, pmm::PAGE_SIZE) {
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

        let pdpt_phys = pml4_entry & PTE_ADDR_MASK;
        if !pmm::is_valid_ram_range(pdpt_phys, pmm::PAGE_SIZE) {
            return None;
        }
        let pdpt = pdpt_phys as *const u64;
        let pdpt_entry = *pdpt.add(pdpt_idx);
        if (pdpt_entry & PAGE_PRESENT) == 0 || (pdpt_entry & PAGE_HUGE) != 0 {
            return None;
        }

        let pd_phys = pdpt_entry & PTE_ADDR_MASK;
        if !pmm::is_valid_ram_range(pd_phys, pmm::PAGE_SIZE) {
            return None;
        }
        let pd = pd_phys as *const u64;
        let pd_entry = *pd.add(pd_idx);
        if (pd_entry & PAGE_PRESENT) == 0 || (pd_entry & PAGE_HUGE) != 0 {
            return None;
        }

        let pt_phys = pd_entry & PTE_ADDR_MASK;
        if !pmm::is_valid_ram_range(pt_phys, pmm::PAGE_SIZE) {
            return None;
        }
        let pt = pt_phys as *mut u64;
        let pt_entry = *pt.add(pt_idx);
        if (pt_entry & PAGE_PRESENT) == 0 {
            return None;
        }

        Some(pt.add(pt_idx))
    }
}

/// Checks whether a virtual address is present in the specified PML4 table.
///
/// # Safety
///
/// Traverses page tables in physical memory.
pub unsafe fn is_page_mapped_in_pml4(pml4_phys: u64, virtual_addr: u64) -> bool {
    get_pte_in_pml4(pml4_phys, virtual_addr).is_some()
}

/// Translates a virtual address to its corresponding physical address within a specific PML4 table.
///
/// # Safety
///
/// Walks the hardware page table structure.
pub unsafe fn get_phys_addr_in_pml4(pml4_phys: u64, virtual_addr: u64) -> Option<u64> {
    #[cfg(test)]
    {
        let _ = (pml4_phys, virtual_addr);
        return None;
    }
    #[cfg(not(test))]
    {
        if pml4_phys == 0 || !pmm::is_valid_ram_range(pml4_phys, pmm::PAGE_SIZE) {
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

        let pdpt_phys = pml4_entry & PTE_ADDR_MASK;
        if !pmm::is_valid_ram_range(pdpt_phys, pmm::PAGE_SIZE) {
            return None;
        }
        let pdpt = pdpt_phys as *const u64;
        let pdpt_entry = *pdpt.add(pdpt_idx);
        if (pdpt_entry & PAGE_PRESENT) == 0 {
            return None;
        }
        if (pdpt_entry & PAGE_HUGE) != 0 {
            return Some(translate_pte_to_phys(pdpt_entry, virtual_addr, 3));
        }

        let pd_phys = pdpt_entry & PTE_ADDR_MASK;
        if !pmm::is_valid_ram_range(pd_phys, pmm::PAGE_SIZE) {
            return None;
        }
        let pd = pd_phys as *const u64;
        let pd_entry = *pd.add(pd_idx);
        if (pd_entry & PAGE_PRESENT) == 0 {
            return None;
        }
        if (pd_entry & PAGE_HUGE) != 0 {
            return Some(translate_pte_to_phys(pd_entry, virtual_addr, 2));
        }

        let pt_phys = pd_entry & PTE_ADDR_MASK;
        if !pmm::is_valid_ram_range(pt_phys, pmm::PAGE_SIZE) {
            return None;
        }
        let pt = pt_phys as *const u64;
        let pt_entry = *pt.add(pt_idx);
        if (pt_entry & PAGE_PRESENT) == 0 {
            return None;
        }

        Some(translate_pte_to_phys(pt_entry, virtual_addr, 1))
    }
}

/// Translates a virtual address to its corresponding physical address within the active address space.
///
/// # Safety
///
/// Traverses active page tables retrieved from CR3.
pub unsafe fn get_phys_addr(virtual_addr: u64) -> Option<u64> {
    get_phys_addr_in_pml4(active_pml4(), virtual_addr)
}
