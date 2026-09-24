// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Architecture-independent Memory Management Unit (MMU) interfaces.
//!
//! Provides abstract contracts for virtual address translation, TLB invalidation,
//! page directory root switching, and architectural page size queries.

/// Generic Memory Management Unit trait for page mapping and translation.
pub trait Mmu {
    /// Virtual address width in bits supported by this processor architecture.
    fn address_bits(&self) -> usize;

    /// Architecture standard physical page size in bytes (typically 4096).
    fn page_size(&self) -> usize;

    /// Invalidates and flushes the Translation Lookaside Buffer (TLB) for a virtual address.
    fn flush_tlb(&self, vaddr: u64);

    /// Invalidates the entire TLB across all executing CPU cores.
    fn flush_tlb_all(&self);

    /// Retrieves the physical base address of the active page table root (e.g. CR3).
    fn active_table_root(&self) -> u64;

    /// Switches the active address space to a new page table root.
    ///
    /// # Safety
    ///
    /// The caller must verify that `root_phys` references a valid, properly mapped
    /// page table hierarchy containing valid identity or higher-half kernel mappings.
    unsafe fn switch_table_root(&self, root_phys: u64);
}
