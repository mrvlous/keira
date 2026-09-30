// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Global VMA tracking table, range validation, and address allocation algorithms.

use super::super::table::{USER_MAX_VADDR, USER_MIN_VADDR};
use super::descriptor::{Vma, MAX_VMAS, MMAP_END, MMAP_START};

pub static mut VMA_TABLE: [Vma; MAX_VMAS] = [Vma::empty(); MAX_VMAS];

/// Cleans up all active VMAs associated with a terminated address space.
///
/// # Safety
///
/// Modifies global kernel VMA tracking structures.
pub unsafe fn cleanup_vmas_for_pml4(pml4_phys: u64) {
    if pml4_phys == 0 {
        return;
    }
    for vma in VMA_TABLE.iter_mut() {
        if vma.is_active && vma.pml4_phys == pml4_phys {
            vma.is_active = false;
        }
    }
}

/// Locates the active VMA enclosing the virtual address within the designated address space.
///
/// # Safety
///
/// Reads global kernel VMA tracking state.
pub unsafe fn find_active_vma(pml4_phys: u64, addr: u64) -> Option<Vma> {
    if pml4_phys == 0 {
        return None;
    }
    for vma in VMA_TABLE.iter() {
        if vma.is_active && vma.pml4_phys == pml4_phys && (vma.start..vma.end).contains(&addr) {
            return Some(*vma);
        }
    }
    None
}

/// Finds the lowest available non-overlapping virtual memory range within `[MMAP_START, MMAP_END)`.
///
/// # Safety
///
/// Inspects global VMA allocation records.
pub unsafe fn find_free_mmap_range(cur_pml4: u64, aligned_len: u64) -> Option<u64> {
    let mut candidate = MMAP_START;
    loop {
        let end = candidate.checked_add(aligned_len)?;
        if end > MMAP_END {
            return None;
        }

        let mut collision = false;
        for vma in VMA_TABLE.iter() {
            if vma.is_active && vma.pml4_phys == cur_pml4 && candidate < vma.end && end > vma.start
            {
                candidate = vma.end;
                collision = true;
                break;
            }
        }

        if !collision {
            return Some(candidate);
        }
    }
}

/// Validates that a requested virtual address range does not overflow and stays within canonical user space.
pub fn validate_virt_addr_range(addr: u64, length: u64) -> Result<(), &'static str> {
    if length == 0 {
        return Ok(());
    }
    let end = match addr.checked_add(length) {
        Some(e) => e,
        None => return Err("Integer overflow in address calculation"),
    };
    if addr >= USER_MIN_VADDR && end <= USER_MAX_VADDR {
        Ok(())
    } else {
        Err("Address range resides outside user space boundaries")
    }
}
