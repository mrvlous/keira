// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! 4-Level x86_64 Page Table traversal, mapping, translation, and page invalidation.

use crate::pmm;
use keira_arch::cpu::{invlpg, read_cr3, write_cr3};

pub const PAGE_PRESENT: u64 = 1 << 0;
pub const PAGE_WRITABLE: u64 = 1 << 1;
pub const PAGE_USER: u64 = 1 << 2;
pub const PAGE_NO_EXECUTE: u64 = 1 << 63;
pub const GB_1_IDENTITY_MAP: u64 = 0x4000_0000;

pub static mut KASLR_SLIDE_OFFSET: u64 = 0x200000;

/// Get the physical base address of the active PML4 table from the CR3 register.
pub unsafe fn active_pml4() -> u64 {
    read_cr3() & !0xFFF
}

/// Switch the active address space by writing a new PML4 physical address to CR3.
pub unsafe fn switch_address_space(pml4_phys: u64) {
    write_cr3(pml4_phys);
}

/// Map a virtual page to a physical frame with specified flags.
pub unsafe fn map_page(
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

    let pml4_addr = active_pml4();
    let pml4 = pml4_addr as *mut u64;

    // 1. Traverse / Allocate PDPT
    let pdpt_entry = *pml4.add(pml4_idx);
    let pdpt_addr = if (pdpt_entry & PAGE_PRESENT) == 0 {
        let frame = pmm::alloc_frame().ok_or("Out of physical memory for PDPT")?;
        *pml4.add(pml4_idx) = frame | PAGE_PRESENT | PAGE_WRITABLE | PAGE_USER;
        frame
    } else {
        if (flags & PAGE_USER) != 0 {
            *pml4.add(pml4_idx) |= PAGE_USER;
        }
        pdpt_entry & !0xFFF
    };
    let pdpt = pdpt_addr as *mut u64;

    // 2. Traverse / Allocate PD
    let pd_entry = *pdpt.add(pdpt_idx);
    let pd_addr = if (pd_entry & PAGE_PRESENT) == 0 {
        let frame = pmm::alloc_frame().ok_or("Out of physical memory for PD")?;
        *pdpt.add(pdpt_idx) = frame | PAGE_PRESENT | PAGE_WRITABLE | PAGE_USER;
        frame
    } else {
        if (flags & PAGE_USER) != 0 {
            *pdpt.add(pdpt_idx) |= PAGE_USER;
        }
        pd_entry & !0xFFF
    };
    let pd = pd_addr as *mut u64;

    // 3. Traverse / Allocate PT
    let pt_entry = *pd.add(pd_idx);
    let pt_addr = if (pt_entry & PAGE_PRESENT) == 0 {
        let frame = pmm::alloc_frame().ok_or("Out of physical memory for PT")?;
        *pd.add(pd_idx) = frame | PAGE_PRESENT | PAGE_WRITABLE | PAGE_USER;
        frame
    } else {
        if (flags & PAGE_USER) != 0 {
            *pd.add(pd_idx) |= PAGE_USER;
        }
        pt_entry & !0xFFF
    };
    let pt = pt_addr as *mut u64;

    // 4. Set PT entry
    *pt.add(pt_idx) = physical_addr | flags | PAGE_PRESENT;

    // 5. Invalidate page in TLB
    invlpg(virtual_addr);

    Ok(())
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
    let pdpt = (pdpt_entry & !0xFFF) as *mut u64;

    let pd_entry = *pdpt.add(pdpt_idx);
    if (pd_entry & PAGE_PRESENT) == 0 {
        return Err("Page not mapped (PD missing)");
    }
    let pd = (pd_entry & !0xFFF) as *mut u64;

    let pt_entry = *pd.add(pd_idx);
    if (pt_entry & PAGE_PRESENT) == 0 {
        return Err("Page not mapped (PT missing)");
    }
    let pt = (pt_entry & !0xFFF) as *mut u64;

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
        unmap_page(virtual_addr)?;
        pmm::free_frame(phys);
        Ok(())
    } else {
        Err("Virtual address not mapped")
    }
}

/// Translate a virtual address to its corresponding physical address.
pub unsafe fn get_phys_addr(virtual_addr: u64) -> Option<u64> {
    let pml4_idx = ((virtual_addr >> 39) & 0x1FF) as usize;
    let pdpt_idx = ((virtual_addr >> 30) & 0x1FF) as usize;
    let pd_idx = ((virtual_addr >> 21) & 0x1FF) as usize;
    let pt_idx = ((virtual_addr >> 12) & 0x1FF) as usize;

    let pml4 = active_pml4() as *const u64;
    let pdpt_entry = *pml4.add(pml4_idx);
    if (pdpt_entry & PAGE_PRESENT) == 0 {
        return None;
    }

    let pdpt = (pdpt_entry & !0xFFF) as *const u64;
    let pd_entry = *pdpt.add(pdpt_idx);
    if (pd_entry & PAGE_PRESENT) == 0 {
        return None;
    }

    let pd = (pd_entry & !0xFFF) as *const u64;
    let pt_entry = *pd.add(pd_idx);
    if (pt_entry & PAGE_PRESENT) == 0 {
        return None;
    }

    let pt = (pt_entry & !0xFFF) as *const u64;
    let entry = *pt.add(pt_idx);
    if (entry & PAGE_PRESENT) == 0 {
        return None;
    }

    Some((entry & !0xFFF) | (virtual_addr & 0xFFF))
}

/// Allocate and map contiguous virtual memory pages for mmap syscall.
pub unsafe fn mmap_anonymous(
    requested_addr: u64,
    len: usize,
    _prot: u64,
) -> Result<u64, &'static str> {
    if len == 0 {
        return Err("Invalid zero length for mmap");
    }
    let pages = len.div_ceil(pmm::PAGE_SIZE as usize);
    let base_vaddr = if requested_addr != 0 && requested_addr >= 0x40000000 {
        requested_addr
    } else {
        0x50000000
    };

    for i in 0..pages {
        let vaddr = base_vaddr + (i as u64 * pmm::PAGE_SIZE);
        let phys_frame = pmm::alloc_frame().ok_or("Out of physical memory for mmap")?;
        map_page(vaddr, phys_frame, PAGE_PRESENT | PAGE_WRITABLE | PAGE_USER)?;
    }

    Ok(base_vaddr)
}

/// Validate if virtual memory range is non-null and page aligned.
pub fn validate_virt_addr_range(vaddr: u64, len: usize) -> bool {
    if vaddr == 0 || len == 0 {
        return false;
    }
    let end = match vaddr.checked_add(len as u64) {
        Some(e) => e,
        None => return false,
    };
    end > vaddr
}

/// Unmap contiguous virtual memory pages for munmap syscall.
pub unsafe fn munmap_pages(vaddr: u64, len: usize) -> Result<(), &'static str> {
    if len == 0 || !vaddr.is_multiple_of(pmm::PAGE_SIZE) || !validate_virt_addr_range(vaddr, len) {
        return Err("Invalid address alignment or length for munmap");
    }
    let pages = len.div_ceil(pmm::PAGE_SIZE as usize);
    for i in 0..pages {
        let addr = vaddr + (i as u64 * pmm::PAGE_SIZE);
        let _ = unmap_page(addr);
    }
    Ok(())
}

/// Adjust protection flags for virtual memory pages (Syscall 31: mprotect).
pub unsafe fn mprotect_pages(vaddr: u64, len: usize, prot: u64) -> Result<(), &'static str> {
    if len == 0 || !vaddr.is_multiple_of(pmm::PAGE_SIZE) || !validate_virt_addr_range(vaddr, len) {
        return Err("Invalid address alignment or length for mprotect");
    }
    let pages = len.div_ceil(pmm::PAGE_SIZE as usize);
    for i in 0..pages {
        let addr = vaddr + (i as u64 * pmm::PAGE_SIZE);
        if let Some(phys) = get_phys_addr(addr) {
            let flags = PAGE_PRESENT | PAGE_USER | if (prot & 2) != 0 { PAGE_WRITABLE } else { 0 };
            let _ = map_page(addr, phys, flags);
        }
    }
    Ok(())
}

/// Advise kernel on memory page management strategies (Syscall 32: madvise).
pub unsafe fn madvise_pages(vaddr: u64, len: usize, _advice: u64) -> Result<(), &'static str> {
    if len == 0 || !vaddr.is_multiple_of(pmm::PAGE_SIZE) || !validate_virt_addr_range(vaddr, len) {
        return Err("Invalid address alignment or length for madvise");
    }
    Ok(())
}

/// Calculate Kernel Address Space Layout Randomization (KASLR) base slide offset.
pub fn get_kaslr_offset() -> u64 {
    unsafe { KASLR_SLIDE_OFFSET }
}
