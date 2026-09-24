// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Page Table Entry (PTE) architecture flags, masks, and address translation routines.

/// Page present bit in page table entries.
pub const PAGE_PRESENT: u64 = 1 << 0;

/// Page writable flag (read/write permission).
pub const PAGE_WRITABLE: u64 = 1 << 1;

/// User/supervisor mode access flag (1 = user-accessible, 0 = supervisor only).
pub const PAGE_USER: u64 = 1 << 2;

/// Page accessed bit set by hardware on read/write.
pub const PAGE_ACCESSED: u64 = 1 << 5;

/// Page dirty bit set by hardware on write.
pub const PAGE_DIRTY: u64 = 1 << 6;

/// Page size bit indicating 2 MiB huge page (PD level) or 1 GiB huge page (PDPT level).
pub const PAGE_HUGE: u64 = 1 << 7;

/// Software Copy-On-Write tracking flag.
pub const PAGE_COW: u64 = 1 << 9;

/// Execute-disable bit preventing instruction fetch from page.
pub const PAGE_NO_EXECUTE: u64 = 1 << 63;

/// 1 GiB physical identity mapping constant.
pub const GB_1_IDENTITY_MAP: u64 = 0x4000_0000;

/// Canonical User Space Virtual Address Lower Boundary (64 KiB null trap guard).
pub const USER_MIN_VADDR: u64 = 0x0000_0000_0001_0000;

/// Upper limit of canonical 47-bit lower half user space addressing.
pub const USER_MAX_VADDR: u64 = 0x0000_7FFF_FFFF_FFFF;

/// Canonical physical address mask for standard 4 KiB page table entries (bits 12..51).
pub const PTE_ADDR_MASK_4K: u64 = 0x000F_FFFF_FFFF_F000;

/// Canonical physical address mask for 2 MiB huge page directory entries (bits 21..51).
pub const PTE_ADDR_MASK_2M: u64 = 0x000F_FFFF_FFE0_0000;

/// Canonical physical address mask for 1 GiB huge page directory pointer entries (bits 30..51).
pub const PTE_ADDR_MASK_1G: u64 = 0x000F_FFFF_C000_0000;

/// Default canonical physical address mask alias for 4 KiB page tables.
pub const PTE_ADDR_MASK: u64 = PTE_ADDR_MASK_4K;

/// Resolves physical address from a page table entry and virtual address given page table level.
///
/// * `level == 3`: 1 GiB huge page at PDPT level.
/// * `level == 2`: 2 MiB huge page at PD level.
/// * `level == 1`: standard 4 KiB page at PT level.
pub fn translate_pte_to_phys(pte: u64, virtual_addr: u64, level: u8) -> u64 {
    match level {
        3 => (pte & PTE_ADDR_MASK_1G) | (virtual_addr & 0x3FFF_FFFF),
        2 => (pte & PTE_ADDR_MASK_2M) | (virtual_addr & 0x1F_FFFF),
        _ => (pte & PTE_ADDR_MASK_4K) | (virtual_addr & 0xFFF),
    }
}

pub static mut KASLR_SLIDE_OFFSET: u64 = 0x200000;

/// Returns current KASLR slide offset.
pub fn get_kaslr_offset() -> u64 {
    unsafe { KASLR_SLIDE_OFFSET }
}
