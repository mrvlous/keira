// SPDX-License-Identifier: GPL-2.0-only
/*
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra <mrvlous@proton.me>
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 */

//! x86_64 Page Table walk routines and page table entry (PTE) manipulation.

pub mod entry {
    pub const PAGE_PRESENT: u64 = 1 << 0;
    
    pub const PAGE_WRITABLE: u64 = 1 << 1;
    
    pub const PAGE_USER: u64 = 1 << 2;
    
    pub const PAGE_ACCESSED: u64 = 1 << 5;
    
    pub const PAGE_DIRTY: u64 = 1 << 6;
    
    pub const PAGE_HUGE: u64 = 1 << 7;
    
    pub const PAGE_COW: u64 = 1 << 9;
    
    pub const PAGE_NO_EXECUTE: u64 = 1 << 63;
    
    pub const GB_1_IDENTITY_MAP: u64 = 0x4000_0000;
    
    pub const USER_MIN_VADDR: u64 = 0x0000_0000_0001_0000;
    
    pub const USER_MAX_VADDR: u64 = 0x0000_7FFF_FFFF_FFFF;
    
    pub const PTE_ADDR_MASK_4K: u64 = 0x000F_FFFF_FFFF_F000;
    
    pub const PTE_ADDR_MASK_2M: u64 = 0x000F_FFFF_FFE0_0000;
    
    pub const PTE_ADDR_MASK_1G: u64 = 0x000F_FFFF_C000_0000;
    
    pub const PTE_ADDR_MASK: u64 = PTE_ADDR_MASK_4K;
    
    pub fn translate_pte_to_phys(pte: u64, virtual_addr: u64, level: u8) -> u64 {
        match level {
            3 => (pte & PTE_ADDR_MASK_1G) | (virtual_addr & 0x3FFF_FFFF),
            2 => (pte & PTE_ADDR_MASK_2M) | (virtual_addr & 0x1F_FFFF),
            _ => (pte & PTE_ADDR_MASK_4K) | (virtual_addr & 0xFFF),
        }
    }
    
    pub static mut KASLR_SLIDE_OFFSET: u64 = 0x200000;
    
    pub fn get_kaslr_offset() -> u64 {
        unsafe { KASLR_SLIDE_OFFSET }
    }
}

pub mod walk {
    use super::entry::PTE_ADDR_MASK;
    #[cfg(not(test))]
    use super::entry::{translate_pte_to_phys, PAGE_HUGE, PAGE_PRESENT};
    #[cfg(not(test))]
    use crate::pmm;
    use keira_arch::cpu::{read_cr3, write_cr3};
    
    static BOOT_PML4_PHYS: core::sync::atomic::AtomicUsize = core::sync::atomic::AtomicUsize::new(0);
    
    pub fn init_boot_pml4() {
        unsafe {
            let pml4 = active_pml4();
            if pml4 != 0 {
                BOOT_PML4_PHYS.store(pml4 as usize, core::sync::atomic::Ordering::SeqCst);
            }
        }
    }
    
    pub fn boot_pml4() -> u64 {
        let p = BOOT_PML4_PHYS.load(core::sync::atomic::Ordering::SeqCst);
        if p != 0 {
            p as u64
        } else {
            unsafe { active_pml4() }
        }
    }
    
    pub unsafe fn active_pml4() -> u64 {
        (read_cr3() as u64) & PTE_ADDR_MASK
    }
    
    pub unsafe fn switch_address_space(pml4_phys: u64) {
        write_cr3(pml4_phys as usize);
    }
    
    pub unsafe fn get_pte_in_pml4(pml4_phys: u64, virtual_addr: u64) -> Option<u64> {
        #[cfg(test)]
        {
            let _ = (pml4_phys, virtual_addr);
            None
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
    
    pub unsafe fn get_pte_mut_in_pml4(pml4_phys: u64, virtual_addr: u64) -> Option<*mut u64> {
        #[cfg(test)]
        {
            let _ = (pml4_phys, virtual_addr);
            None
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
    
    pub unsafe fn is_page_mapped_in_pml4(pml4_phys: u64, virtual_addr: u64) -> bool {
        get_pte_in_pml4(pml4_phys, virtual_addr).is_some()
    }
    
    pub unsafe fn get_phys_addr_in_pml4(pml4_phys: u64, virtual_addr: u64) -> Option<u64> {
        #[cfg(test)]
        {
            let _ = (pml4_phys, virtual_addr);
            None
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
    
    pub unsafe fn get_phys_addr(virtual_addr: u64) -> Option<u64> {
        get_phys_addr_in_pml4(active_pml4(), virtual_addr)
    }
}

pub use self::entry::*;
pub use self::walk::*;
