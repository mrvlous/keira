// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! PML4 Address space cloning preserving kernel identity map, MMIO, and user space memory.

use super::paging::{
    active_pml4, map_page_in_pml4, switch_address_space, PAGE_NO_EXECUTE, PAGE_PRESENT, PAGE_USER,
    PAGE_WRITABLE,
};
use crate::pmm;

/// Clone the boot PML4, sharing kernel identity-map (PDPT[0]) and MMIO (PDPT[3]).
/// User-space entries are left empty for a new process to populate.
pub unsafe fn clone_kernel_pml4() -> Result<u64, &'static str> {
    let boot_pml4_phys = active_pml4();
    let boot_pml4 = boot_pml4_phys as *const u64;

    // Allocate a new PML4 frame (zeroed by pmm::alloc_frame)
    let new_pml4_phys = pmm::alloc_frame().ok_or("Out of memory for new PML4")?;
    let new_pml4 = new_pml4_phys as *mut u64;

    // Read the boot PML4[0] entry - it points to the boot PDPT
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

    // Copy kernel identity map (PDPT[0]: 0..1GB) and kernel MMIO/Framebuffer (PDPT[3]: 3..4GB)
    *new_pdpt.add(0) = *boot_pdpt.add(0);
    *new_pdpt.add(3) = *boot_pdpt.add(3);

    // Set new PML4[0] = new PDPT with present + writable + user flags
    *new_pml4 = new_pdpt_phys | PAGE_PRESENT | PAGE_WRITABLE | PAGE_USER;

    Ok(new_pml4_phys)
}

/// Deep clone an entire parent process address space (kernel mappings + user pages).
/// Allocates separate physical frames for every mapped user page to guarantee isolation.
pub unsafe fn clone_user_address_space(parent_pml4_phys: u64) -> Result<u64, &'static str> {
    let child_pml4_phys = clone_kernel_pml4()?;

    let current_pml4 = active_pml4();
    switch_address_space(parent_pml4_phys);

    let parent_pml4 = parent_pml4_phys as *const u64;

    // Walk all PML4 entries
    for pml4_idx in 0..512 {
        let pml4_entry = *parent_pml4.add(pml4_idx);
        if (pml4_entry & PAGE_PRESENT) == 0 {
            continue;
        }

        let pdpt_phys = pml4_entry & !0xFFF;
        let pdpt = pdpt_phys as *const u64;

        for pdpt_idx in 0..512 {
            // Skip kernel identity map (pml4_idx=0, pdpt_idx=0) and MMIO (pml4_idx=0, pdpt_idx=3)
            if pml4_idx == 0 && (pdpt_idx == 0 || pdpt_idx == 3) {
                continue;
            }

            let pdpt_entry = *pdpt.add(pdpt_idx);
            if (pdpt_entry & PAGE_PRESENT) == 0 {
                continue;
            }

            let pd_phys = pdpt_entry & !0xFFF;
            let pd = pd_phys as *const u64;

            for pd_idx in 0..512 {
                let pd_entry = *pd.add(pd_idx);
                if (pd_entry & PAGE_PRESENT) == 0 {
                    continue;
                }

                let pt_phys = pd_entry & !0xFFF;
                let pt = pt_phys as *const u64;

                for pt_idx in 0..512 {
                    let pt_entry = *pt.add(pt_idx);
                    if (pt_entry & PAGE_PRESENT) == 0 || (pt_entry & PAGE_USER) == 0 {
                        continue;
                    }

                    // Calculate virtual address
                    let mut vaddr = 0u64;
                    vaddr |= (pml4_idx as u64) << 39;
                    vaddr |= (pdpt_idx as u64) << 30;
                    vaddr |= (pd_idx as u64) << 21;
                    vaddr |= (pt_idx as u64) << 12;

                    // Allocate fresh frame for child
                    let child_frame = match pmm::alloc_frame() {
                        Some(f) => f,
                        None => {
                            switch_address_space(current_pml4);
                            return Err("Out of physical memory cloning user page");
                        }
                    };

                    let page_flags =
                        (pt_entry & (PAGE_USER | PAGE_WRITABLE | PAGE_NO_EXECUTE)) | PAGE_PRESENT;

                    // Map in child PML4
                    if let Err(e) =
                        map_page_in_pml4(child_pml4_phys, vaddr, child_frame, page_flags)
                    {
                        pmm::free_frame(child_frame);
                        switch_address_space(current_pml4);
                        return Err(e);
                    }

                    // Copy parent page content to child frame
                    let src_ptr = vaddr as *const u8;
                    let dst_ptr = child_frame as *mut u8;
                    core::ptr::copy_nonoverlapping(src_ptr, dst_ptr, pmm::PAGE_SIZE as usize);
                }
            }
        }
    }

    switch_address_space(current_pml4);
    Ok(child_pml4_phys)
}
