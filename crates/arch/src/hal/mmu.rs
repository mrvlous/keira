// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Architecture-independent Memory Management Unit (MMU) traits.

/// Generic Memory Management Unit trait for page mapping and translation.
pub trait Mmu {
    /// Virtual address size in bits supported by this architecture.
    fn address_bits(&self) -> usize;

    /// Architecture standard physical page size in bytes.
    fn page_size(&self) -> usize;

    /// Invalidate/flush translation lookaside buffer (TLB) for a virtual address.
    fn flush_tlb(&self, vaddr: u64);

    /// Invalidate entire TLB across all CPU cores.
    fn flush_tlb_all(&self);

    /// Retrieve the physical address of the active page table root.
    fn active_table_root(&self) -> u64;

    /// Switch active address space to a new page table root.
    ///
    /// # Safety
    /// Caller must ensure the target page table contains valid kernel mappings.
    unsafe fn switch_table_root(&self, root_phys: u64);
}
