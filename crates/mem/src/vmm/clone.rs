// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! PML4 Address space cloning preserving kernel identity map and MMIO/framebuffer.

use super::paging::{active_pml4, PAGE_PRESENT, PAGE_USER, PAGE_WRITABLE};
use crate::pmm;

/// Clone the boot PML4, sharing kernel identity-map (PDPT[0]) and MMIO (PDPT[3]).
/// User-space entries are left empty for the new process to populate.
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

    // PDPT[1..2] are zeroed by alloc_frame - user space starts fresh

    // Set new PML4[0] = new PDPT with present + writable + user flags
    *new_pml4 = new_pdpt_phys | PAGE_PRESENT | PAGE_WRITABLE | PAGE_USER;

    // PML4[1..511] are zeroed by alloc_frame - user heap/stack space starts fresh

    Ok(new_pml4_phys)
}
