// SPDX-License-Identifier: GPL-2.0-only

//! Global VMA tracking table, range validation and address allocation algorithms.

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

/// Optional callback hook providing per-task ASLR offset for dynamic mmap allocations.
static mut MMAP_ASLR_OFFSET_HOOK: Option<fn() -> u64> = None;

/// Registers a callback hook providing per-task ASLR offset for mmap ranges.
pub fn register_mmap_aslr_hook(hook: fn() -> u64) {
    unsafe {
        MMAP_ASLR_OFFSET_HOOK = Some(hook);
    }
}

/// Finds the lowest available non-overlapping virtual memory range within `[MMAP_START, MMAP_END)`.
///
/// When an ASLR hook is registered and active, begins search from `MMAP_START + offset`
/// and wraps around to `MMAP_START` if no contiguous space is found at the randomized offset.
///
/// # Safety
///
/// Inspects global VMA allocation records.
pub unsafe fn find_free_mmap_range(cur_pml4: u64, aligned_len: u64) -> Option<u64> {
    let offset = MMAP_ASLR_OFFSET_HOOK.map(|f| f()).unwrap_or(0);
    let start_candidate = match MMAP_START.checked_add(offset) {
        Some(s) if s < MMAP_END => s,
        _ => MMAP_START,
    };

    let mut candidate = start_candidate;
    let mut wrapped = false;

    loop {
        let end = candidate.checked_add(aligned_len)?;
        if end > MMAP_END {
            if !wrapped && start_candidate > MMAP_START {
                candidate = MMAP_START;
                wrapped = true;
                continue;
            }
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

/// Copies active VMAs associated with a given address space into the destination slice.
///
/// # Safety
///
/// Reads global kernel VMA tracking structures.
pub unsafe fn get_vmas_for_pml4(pml4_phys: u64, out_vmas: &mut [Vma]) -> usize {
    if pml4_phys == 0 {
        return 0;
    }
    let mut count = 0;
    for vma in VMA_TABLE.iter() {
        if vma.is_active && vma.pml4_phys == pml4_phys {
            if count < out_vmas.len() {
                out_vmas[count] = *vma;
                count += 1;
            }
        }
    }
    count
}

/// Clones all active VMAs from parent address space to child address space.
///
/// # Safety
///
/// Modifies global kernel VMA tracking structures.
pub unsafe fn clone_vmas_for_pml4(parent_pml4: u64, child_pml4: u64) -> Result<(), &'static str> {
    if parent_pml4 == 0 || child_pml4 == 0 {
        return Ok(());
    }
    for i in 0..MAX_VMAS {
        let vma = VMA_TABLE[i];
        if vma.is_active && vma.pml4_phys == parent_pml4 {
            let mut slot = None;
            for (j, candidate) in VMA_TABLE.iter().enumerate() {
                if !candidate.is_active {
                    slot = Some(j);
                    break;
                }
            }
            let free_slot = slot.ok_or("Max VMA capacity reached during fork clone")?;
            let mut child_vma = vma;
            child_vma.pml4_phys = child_pml4;
            VMA_TABLE[free_slot] = child_vma;
        }
    }
    Ok(())
}

/// Retrieves total number of active VMAs currently tracked across the kernel.
///
/// # Safety
///
/// Reads global kernel VMA tracking structures.
pub unsafe fn total_active_vmas() -> usize {
    let mut count = 0;
    for vma in VMA_TABLE.iter() {
        if vma.is_active {
            count += 1;
        }
    }
    count
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
