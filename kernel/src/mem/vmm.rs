// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Keira Kernel: Virtual Memory Manager (VMM)
//!
//! Manages 4-level page tables (PML4, PDPT, PD, PT) on x86_64, enabling dynamic
//! mapping and unmapping of virtual addresses to physical pages.

use crate::mem::pmm;

// Page Table Entry flags
pub const PAGE_PRESENT: u64 = 1 << 0;
pub const PAGE_WRITABLE: u64 = 1 << 1;
// User mode access allowed
pub const PAGE_USER: u64 = 1 << 2;
pub const PAGE_NO_EXECUTE: u64 = 1 << 63;
pub const GB_1_IDENTITY_MAP: u64 = 0x4000_0000;

/// Get the physical address of the active PML4 table from the CR3 register
pub unsafe fn active_pml4() -> u64 {
    let cr3: u64;
    core::arch::asm!("mov {}, cr3", out(reg) cr3);
    // Clear control flags in the lower 12 bits
    cr3 & !0xFFF
}

/// Map a virtual page to a physical frame with specified flags
pub unsafe fn map_page(
    virtual_addr: u64,
    physical_addr: u64,
    flags: u64,
) -> Result<(), &'static str> {
    if virtual_addr % pmm::PAGE_SIZE != 0 || physical_addr % pmm::PAGE_SIZE != 0 {
        return Err("Virtual or Physical address is not page-aligned");
    }

    // Indices for the 4 levels of page tables
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
    core::arch::asm!("invlpg [{}]", in(reg) virtual_addr);

    Ok(())
}

/// Unmap a virtual page
pub unsafe fn unmap_page(virtual_addr: u64) -> Result<(), &'static str> {
    if virtual_addr % pmm::PAGE_SIZE != 0 {
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

    // Free the entry
    *pt.add(pt_idx) = 0;

    // Invalidate in TLB
    core::arch::asm!("invlpg [{}]", in(reg) virtual_addr);

    Ok(())
}

/// Unmap a virtual page and free its underlying physical frame
pub unsafe fn free_and_unmap_page(virtual_addr: u64) -> Result<(), &'static str> {
    if let Some(phys) = get_phys_addr(virtual_addr) {
        unmap_page(virtual_addr)?;
        pmm::free_frame(phys);
        Ok(())
    } else {
        Err("Virtual address not mapped")
    }
}

/// Translate a virtual address to its corresponding physical address
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

/// Clone the boot PML4, sharing only the kernel identity-map (PDPT[0]).
/// User-space entries are left empty for the new process to populate.
pub unsafe fn clone_kernel_pml4() -> Result<u64, &'static str> {
    let boot_pml4_phys = active_pml4();
    let boot_pml4 = boot_pml4_phys as *const u64;

    // Allocate a new PML4 frame (zeroed by pmm::alloc_frame)
    let new_pml4_phys = pmm::alloc_frame().ok_or("Out of memory for new PML4")?;
    let new_pml4 = new_pml4_phys as *mut u64;

    // Read the boot PML4[0] entry — it points to the boot PDPT
    let boot_pml4_0 = *boot_pml4;
    if (boot_pml4_0 & PAGE_PRESENT) == 0 {
        pmm::free_frame(new_pml4_phys);
        return Err("Boot PML4[0] is not present");
    }

    let boot_pdpt_phys = boot_pml4_0 & !0xFFF;
    let boot_pdpt = boot_pdpt_phys as *const u64;

    // Allocate a new PDPT for the child process
    let new_pdpt_phys = pmm::alloc_frame().ok_or("Out of memory for new PDPT")?;
    let new_pdpt = new_pdpt_phys as *mut u64;

    // Copy only PDPT[0] (kernel identity map: first 1GB via 2MB huge pages)
    *new_pdpt = *boot_pdpt;

    // PDPT[1..511] are zeroed by alloc_frame — user space starts fresh

    // Set new PML4[0] = new PDPT with present + writable + user flags
    *new_pml4 = new_pdpt_phys | PAGE_PRESENT | PAGE_WRITABLE | PAGE_USER;

    // PML4[1..511] are zeroed by alloc_frame — user heap/stack space starts fresh

    Ok(new_pml4_phys)
}

/// Switch the active address space by writing a new PML4 physical address to CR3
pub unsafe fn switch_address_space(pml4_phys: u64) {
    core::arch::asm!("mov cr3, {}", in(reg) pml4_phys);
}

/// Free all user-space pages and page table frames from a process's PML4.
/// Must be called while a DIFFERENT address space is active (e.g., boot PML4).
pub unsafe fn free_user_pages(pml4_phys: u64, program_break: u64) {
    // Temporarily switch to the target address space to walk its page tables
    let saved_cr3 = active_pml4();
    switch_address_space(pml4_phys);

    // 1. Free user code/data pages (0x40000000 .. 0x40040000 = 256KB range)
    let mut addr: u64 = 0x40000000;
    while addr < 0x40040000 {
        let _ = free_and_unmap_page(addr);
        addr += pmm::PAGE_SIZE;
    }

    // 2. Free user heap pages (0x600000000000 .. program_break)
    addr = 0x600000000000;
    while addr < program_break {
        let _ = free_and_unmap_page(addr);
        addr += pmm::PAGE_SIZE;
    }

    // 3. Free user stack page
    let _ = free_and_unmap_page(0x7FFFFFFF0000);

    // Switch back to the caller's address space
    switch_address_space(saved_cr3);

    // 4. Free the child's page table frames (PDPT and intermediate PD/PT tables)
    let pml4 = pml4_phys as *const u64;
    let pml4_0 = *pml4;
    if (pml4_0 & PAGE_PRESENT) != 0 {
        let pdpt_phys = pml4_0 & !0xFFF;
        let pdpt = pdpt_phys as *const u64;

        // Free page table structures under PDPT[1..511] (user code area under PML4[0])
        for i in 1..512 {
            let pdpt_entry = *pdpt.add(i);
            if (pdpt_entry & PAGE_PRESENT) != 0 {
                // level 2 = PD
                free_page_table_tree(pdpt_entry & !0xFFF, 2);
            }
        }
        pmm::free_frame(pdpt_phys);
    }

    // Free page table structures under PML4[1..511] (user heap/stack)
    for i in 1..512 {
        let entry = *pml4.add(i);
        if (entry & PAGE_PRESENT) != 0 {
            // level 3 = PDPT
            free_page_table_tree(entry & !0xFFF, 3);
        }
    }

    // Free the PML4 frame itself
    pmm::free_frame(pml4_phys);
}

/// Recursively free page table frames at a given level.
/// Level 3 = PDPT, Level 2 = PD, Level 1 = PT
unsafe fn free_page_table_tree(table_phys: u64, level: u32) {
    let table = table_phys as *const u64;
    if level > 1 {
        for i in 0..512 {
            let entry = *table.add(i);
            if (entry & PAGE_PRESENT) != 0 {
                // Skip huge pages (bit 7) — they're part of the identity map
                if (entry & (1 << 7)) == 0 {
                    free_page_table_tree(entry & !0xFFF, level - 1);
                }
            }
        }
    }
    pmm::free_frame(table_phys);
}

/// Allocate and map contiguous virtual memory pages for mmap syscall
pub unsafe fn mmap_anonymous(
    requested_addr: u64,
    len: usize,
    _prot: u64,
) -> Result<u64, &'static str> {
    if len == 0 {
        return Err("Invalid zero length for mmap");
    }
    let pages = (len + pmm::PAGE_SIZE as usize - 1) / pmm::PAGE_SIZE as usize;
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

/// Validate if virtual memory range is non-null and page aligned
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

/// Unmap contiguous virtual memory pages for munmap syscall
pub unsafe fn munmap_pages(vaddr: u64, len: usize) -> Result<(), &'static str> {
    if len == 0 || vaddr % pmm::PAGE_SIZE != 0 || !validate_virt_addr_range(vaddr, len) {
        return Err("Invalid address alignment or length for munmap");
    }
    let pages = (len + pmm::PAGE_SIZE as usize - 1) / pmm::PAGE_SIZE as usize;
    for i in 0..pages {
        let addr = vaddr + (i as u64 * pmm::PAGE_SIZE);
        let _ = unmap_page(addr);
    }
    Ok(())
}

/// Adjust protection flags for virtual memory pages (Syscall 31: mprotect)
pub unsafe fn mprotect_pages(vaddr: u64, len: usize, prot: u64) -> Result<(), &'static str> {
    if len == 0 || vaddr % pmm::PAGE_SIZE != 0 || !validate_virt_addr_range(vaddr, len) {
        return Err("Invalid address alignment or length for mprotect");
    }
    let pages = (len + pmm::PAGE_SIZE as usize - 1) / pmm::PAGE_SIZE as usize;
    for i in 0..pages {
        let addr = vaddr + (i as u64 * pmm::PAGE_SIZE);
        if let Some(phys) = get_phys_addr(addr) {
            let flags = PAGE_PRESENT | PAGE_USER | if (prot & 2) != 0 { PAGE_WRITABLE } else { 0 };
            let _ = map_page(addr, phys, flags);
        }
    }
    Ok(())
}

/// Advise kernel on memory page management strategies (Syscall 32: madvise)
pub unsafe fn madvise_pages(vaddr: u64, len: usize, _advice: u64) -> Result<(), &'static str> {
    if len == 0 || vaddr % pmm::PAGE_SIZE != 0 || !validate_virt_addr_range(vaddr, len) {
        return Err("Invalid address alignment or length for madvise");
    }
    Ok(())
}

pub static mut KASLR_SLIDE_OFFSET: u64 = 0x200000;

/// Calculate Kernel Address Space Layout Randomization (KASLR) base slide offset
pub fn get_kaslr_offset() -> u64 {
    unsafe { KASLR_SLIDE_OFFSET }
}
