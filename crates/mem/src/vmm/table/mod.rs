// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Page table architectural structures, entry bitmasks, and hardware traversal.

pub mod entry;
pub mod walk;

pub use entry::{
    get_kaslr_offset, translate_pte_to_phys, GB_1_IDENTITY_MAP, KASLR_SLIDE_OFFSET, PAGE_ACCESSED,
    PAGE_COW, PAGE_DIRTY, PAGE_HUGE, PAGE_NO_EXECUTE, PAGE_PRESENT, PAGE_USER, PAGE_WRITABLE,
    PTE_ADDR_MASK, PTE_ADDR_MASK_1G, PTE_ADDR_MASK_2M, PTE_ADDR_MASK_4K, USER_MAX_VADDR,
    USER_MIN_VADDR,
};
pub use walk::{
    active_pml4, get_phys_addr, get_phys_addr_in_pml4, get_pte_in_pml4, get_pte_mut_in_pml4,
    is_page_mapped_in_pml4, switch_address_space,
};
