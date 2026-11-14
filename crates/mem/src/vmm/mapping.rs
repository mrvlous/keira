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

//! Virtual memory page mapping, protection modifications and unmapping.

pub mod map {
    use super::super::table::{
        active_pml4, PAGE_HUGE, PAGE_PRESENT, PAGE_USER, PAGE_WRITABLE, PTE_ADDR_MASK, PTE_ADDR_MASK_2M,
    };
    use crate::pmm;
    use keira_arch::cpu::invlpg;
    
    pub unsafe fn map_page(
        virtual_addr: u64,
        physical_addr: u64,
        flags: u64,
    ) -> Result<(), &'static str> {
        map_page_in_pml4(active_pml4(), virtual_addr, physical_addr, flags)
    }
    
    pub unsafe fn map_page_in_pml4(
        pml4_phys: u64,
        virtual_addr: u64,
        physical_addr: u64,
        flags: u64,
    ) -> Result<(), &'static str> {
        if pml4_phys == 0 {
            return Ok(());
        }
        if !virtual_addr.is_multiple_of(pmm::PAGE_SIZE) || !physical_addr.is_multiple_of(pmm::PAGE_SIZE)
        {
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
            core::ptr::write_bytes(frame as *mut u8, 0, 4096);
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
                Some(f) => {
                    core::ptr::write_bytes(f as *mut u8, 0, 4096);
                    f
                }
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
                Some(f) => {
                    core::ptr::write_bytes(f as *mut u8, 0, 4096);
                    f
                }
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
    
    pub unsafe fn map_huge_2m_page(
        virtual_addr: u64,
        physical_addr: u64,
        flags: u64,
    ) -> Result<(), &'static str> {
        if !virtual_addr.is_multiple_of(0x20_0000) {
            return Err("Virtual address is not 2MB aligned");
        }
        if !physical_addr.is_multiple_of(0x20_0000) {
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
            core::ptr::write_bytes(frame as *mut u8, 0, 4096);
            *pml4.add(pml4_idx) = frame | PAGE_PRESENT | PAGE_WRITABLE | PAGE_USER;
            frame
        } else {
            pml4_entry & PTE_ADDR_MASK
        };
        let pdpt = pdpt_addr as *mut u64;
    
        let pdpt_entry = *pdpt.add(pdpt_idx);
        let pd_addr = if (pdpt_entry & PAGE_PRESENT) == 0 {
            let frame = pmm::alloc_frame().ok_or("Out of physical memory for PD")?;
            core::ptr::write_bytes(frame as *mut u8, 0, 4096);
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
}

pub mod protect {
    use super::super::table::{
        active_pml4, PAGE_COW, PAGE_HUGE, PAGE_PRESENT, PAGE_USER, PAGE_WRITABLE, PTE_ADDR_MASK,
        PTE_ADDR_MASK_1G, PTE_ADDR_MASK_2M,
    };
    use crate::pmm;
    use keira_arch::interrupts::smp::tlb_shootdown;
    
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
                tlb_shootdown(virtual_addr);
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
                tlb_shootdown(virtual_addr);
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
    
        tlb_shootdown(virtual_addr);
        Ok(())
    }
    
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
}

pub mod unmap {
    use super::super::table::{
        active_pml4, get_pte_in_pml4, PAGE_HUGE, PAGE_PRESENT, PAGE_USER, PTE_ADDR_MASK,
        PTE_ADDR_MASK_1G, PTE_ADDR_MASK_2M,
    };
    use crate::pmm;
    use keira_arch::interrupts::smp::tlb_shootdown;
    
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
                tlb_shootdown(virtual_addr);
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
                tlb_shootdown(virtual_addr);
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
        tlb_shootdown(virtual_addr);
    
        Ok(())
    }
    
    pub unsafe fn unmap_huge_2m_page(virtual_addr: u64) -> Result<(), &'static str> {
        if !virtual_addr.is_multiple_of(0x20_0000) {
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
        tlb_shootdown(virtual_addr);
        Ok(())
    }
    
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
                    let frame_count = if is_1gb { 512 * 512 } else { 512 };
                    pmm::free_contiguous_frames(frame, frame_count);
                }
                return Ok(());
            }
            let frame = entry & PTE_ADDR_MASK;
            unmap_page(virtual_addr)?;
            if frame >= pmm::KERNEL_BASE_1MB {
                pmm::release_frame(frame);
            }
            Ok(())
        } else {
            Err("Virtual address not mapped")
        }
    }
}

pub use self::map::*;
pub use self::protect::*;
pub use self::unmap::*;
