// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! User virtual memory area (VMA) allocator, per-address-space VMA tracking, mmap, partial munmap, and partial mprotect.

use super::paging::{
    active_pml4, free_and_unmap_page, map_page, mprotect_page, PAGE_NO_EXECUTE, PAGE_PRESENT,
    PAGE_USER, PAGE_WRITABLE,
};
use crate::pmm;

pub const PROT_NONE: u32 = 0;
pub const PROT_READ: u32 = 1 << 0;
pub const PROT_WRITE: u32 = 1 << 1;
pub const PROT_EXEC: u32 = 1 << 2;
pub const SUPPORTED_PROT: u32 = PROT_READ | PROT_WRITE | PROT_EXEC;

pub const MAP_SHARED: u32 = 1 << 0;
pub const MAP_PRIVATE: u32 = 1 << 1;
pub const MAP_FIXED: u32 = 1 << 4;
pub const MAP_ANONYMOUS: u32 = 1 << 5;

pub const MMAP_START: u64 = 0x5000_0000_0000;
pub const MMAP_END: u64 = 0x7000_0000_0000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Vma {
    pub pml4_phys: u64,
    pub start: u64,
    pub end: u64,
    pub prot: u32,
    pub flags: u32,
    pub is_active: bool,
}

pub const MAX_VMAS: usize = 64;
static mut VMA_TABLE: [Vma; MAX_VMAS] = [Vma {
    pml4_phys: 0,
    start: 0,
    end: 0,
    prot: 0,
    flags: 0,
    is_active: false,
}; MAX_VMAS];

/// Clean up all active VMAs belonging to a specific address space.
pub unsafe fn cleanup_vmas_for_pml4(pml4_phys: u64) {
    if pml4_phys == 0 {
        return;
    }
    for i in 0..MAX_VMAS {
        if VMA_TABLE[i].is_active && VMA_TABLE[i].pml4_phys == pml4_phys {
            VMA_TABLE[i].is_active = false;
        }
    }
}

/// Find the lowest available non-overlapping virtual memory range within [MMAP_START, MMAP_END)
/// for the specified address space (cur_pml4).
pub unsafe fn find_free_mmap_range(cur_pml4: u64, aligned_len: u64) -> Option<u64> {
    let mut candidate = MMAP_START;
    loop {
        let end = match candidate.checked_add(aligned_len) {
            Some(e) => e,
            None => return None,
        };
        if end > MMAP_END {
            return None;
        }

        let mut collision = false;
        for i in 0..MAX_VMAS {
            let vma = VMA_TABLE[i];
            if vma.is_active && vma.pml4_phys == cur_pml4 {
                if candidate < vma.end && end > vma.start {
                    candidate = vma.end;
                    collision = true;
                    break;
                }
            }
        }

        if !collision {
            return Some(candidate);
        }
    }
}

/// Allocate and map an anonymous virtual memory region for user space with per-process VMA bookkeeping.
pub unsafe fn sys_mmap(
    hint_addr: u64,
    length: u64,
    prot: u32,
    flags: u32,
) -> Result<u64, &'static str> {
    if length == 0 {
        return Err("Invalid mmap length (0)");
    }

    // Validate protection bits
    if (prot & !SUPPORTED_PROT) != 0 {
        return Err("Invalid protection flags (EINVAL)");
    }

    // Strict W^X policy enforcement
    if (prot & PROT_WRITE) != 0 && (prot & PROT_EXEC) != 0 {
        return Err("W^X violation: simultaneous PROT_WRITE and PROT_EXEC prohibited");
    }

    let aligned_len = match length.checked_add(pmm::PAGE_SIZE - 1) {
        Some(l) => l & !(pmm::PAGE_SIZE - 1),
        None => return Err("Integer overflow calculating mmap length"),
    };

    let cur_pml4 = active_pml4();

    let start_vaddr = if (flags & MAP_FIXED) != 0 {
        if hint_addr < MMAP_START || hint_addr >= MMAP_END {
            return Err("MAP_FIXED address outside user mmap region");
        }
        if !hint_addr.is_multiple_of(pmm::PAGE_SIZE) {
            return Err("MAP_FIXED address is not page-aligned");
        }
        let fixed_end = match hint_addr.checked_add(aligned_len) {
            Some(e) => e,
            None => return Err("Integer overflow in MAP_FIXED range"),
        };
        if fixed_end > MMAP_END {
            return Err("MAP_FIXED range exceeds user mmap boundary");
        }

        // Check for collision with existing active VMAs in the same address space
        for i in 0..MAX_VMAS {
            let vma = VMA_TABLE[i];
            if vma.is_active
                && vma.pml4_phys == cur_pml4
                && !(fixed_end <= vma.start || hint_addr >= vma.end)
            {
                return Err("MAP_FIXED collision with existing VMA mapping");
            }
        }
        hint_addr
    } else {
        match find_free_mmap_range(cur_pml4, aligned_len) {
            Some(vaddr) => vaddr,
            None => return Err("Out of virtual address space in mmap region"),
        }
    };

    // Find and reserve free VMA slot before touching physical memory or page tables
    let mut vma_slot = None;
    for i in 0..MAX_VMAS {
        if !VMA_TABLE[i].is_active {
            vma_slot = Some(i);
            break;
        }
    }
    let slot_idx = vma_slot.ok_or("Max process VMA mapping table capacity reached")?;

    let mut page_flags = PAGE_USER | PAGE_PRESENT;
    if (prot & PROT_WRITE) != 0 {
        page_flags |= PAGE_WRITABLE;
    }
    if (prot & PROT_EXEC) == 0 {
        page_flags |= PAGE_NO_EXECUTE;
    }

    let mut allocated_pages = 0u64;
    while allocated_pages < aligned_len {
        let vaddr = start_vaddr + allocated_pages;
        let frame = match pmm::alloc_frame() {
            Some(f) => f,
            None => {
                // Rollback previously mapped pages
                let mut rollback_offset = 0u64;
                while rollback_offset < allocated_pages {
                    let _ = free_and_unmap_page(start_vaddr + rollback_offset);
                    rollback_offset += pmm::PAGE_SIZE;
                }
                return Err("Out of physical memory during mmap allocation");
            }
        };

        if let Err(e) = map_page(vaddr, frame, page_flags) {
            pmm::free_frame(frame);
            let mut rollback_offset = 0u64;
            while rollback_offset < allocated_pages {
                let _ = free_and_unmap_page(start_vaddr + rollback_offset);
                rollback_offset += pmm::PAGE_SIZE;
            }
            return Err(e);
        }

        let ptr = vaddr as *mut u8;
        core::ptr::write_bytes(ptr, 0, pmm::PAGE_SIZE as usize);
        allocated_pages += pmm::PAGE_SIZE;
    }

    // Record authoritative VMA tagged with active PML4 ownership
    VMA_TABLE[slot_idx] = Vma {
        pml4_phys: cur_pml4,
        start: start_vaddr,
        end: start_vaddr + aligned_len,
        prot,
        flags,
        is_active: true,
    };

    Ok(start_vaddr)
}

/// Unmap a user virtual memory region and release underlying physical frames with VMA splitting.
pub unsafe fn sys_munmap(addr: u64, length: u64) -> Result<(), &'static str> {
    if length == 0 || !addr.is_multiple_of(pmm::PAGE_SIZE) {
        return Err("Invalid address alignment or zero length for munmap");
    }

    if addr < MMAP_START || addr >= MMAP_END {
        return Err("Address outside user mmap region");
    }

    let aligned_len = match length.checked_add(pmm::PAGE_SIZE - 1) {
        Some(l) => l & !(pmm::PAGE_SIZE - 1),
        None => return Err("Integer overflow in munmap length"),
    };

    let target_end = match addr.checked_add(aligned_len) {
        Some(e) => e,
        None => return Err("Integer overflow in target unmap range"),
    };

    let cur_pml4 = active_pml4();

    // Verify range resides inside an active VMA belonging to current address space
    let mut matching_vma = None;
    for i in 0..MAX_VMAS {
        let vma = VMA_TABLE[i];
        if vma.is_active && vma.pml4_phys == cur_pml4 && addr >= vma.start && target_end <= vma.end
        {
            matching_vma = Some(i);
            break;
        }
    }

    let vma_idx = matching_vma.ok_or("No matching active user VMA for munmap range")?;
    let orig_vma = VMA_TABLE[vma_idx];

    // If middle split is required, verify that a free VMA slot exists before modifying state
    let is_middle_split = addr > orig_vma.start && target_end < orig_vma.end;
    let split_slot = if is_middle_split {
        let mut free_slot = None;
        for i in 0..MAX_VMAS {
            if !VMA_TABLE[i].is_active {
                free_slot = Some(i);
                break;
            }
        }
        Some(free_slot.ok_or("Max VMA capacity reached during partial munmap split")?)
    } else {
        None
    };

    // Unmap physical pages
    let mut offset = 0u64;
    while offset < aligned_len {
        let vaddr = addr + offset;
        let _ = free_and_unmap_page(vaddr);
        offset += pmm::PAGE_SIZE;
    }

    // Update VMA metadata: exact match, front trim, back trim, or middle split
    if addr == orig_vma.start && target_end == orig_vma.end {
        // Case 1: Exact match -> deactivate
        VMA_TABLE[vma_idx].is_active = false;
    } else if addr == orig_vma.start && target_end < orig_vma.end {
        // Case 2: Front trim -> shrink start
        VMA_TABLE[vma_idx].start = target_end;
    } else if addr > orig_vma.start && target_end == orig_vma.end {
        // Case 3: Back trim -> shrink end
        VMA_TABLE[vma_idx].end = addr;
    } else if let Some(new_idx) = split_slot {
        // Case 4: Middle split -> left half stays in vma_idx, right half in new_idx
        VMA_TABLE[vma_idx].end = addr;
        VMA_TABLE[new_idx] = Vma {
            pml4_phys: orig_vma.pml4_phys,
            start: target_end,
            end: orig_vma.end,
            prot: orig_vma.prot,
            flags: orig_vma.flags,
            is_active: true,
        };
    }

    Ok(())
}

/// Change protection flags on an existing mapped user memory range with VMA splitting.
pub unsafe fn sys_mprotect(addr: u64, length: u64, prot: u32) -> Result<(), &'static str> {
    if length == 0 || !addr.is_multiple_of(pmm::PAGE_SIZE) {
        return Err("Invalid address alignment or zero length for mprotect");
    }

    if (prot & !SUPPORTED_PROT) != 0 {
        return Err("Invalid protection flags (EINVAL)");
    }

    // Strict W^X policy enforcement
    if (prot & PROT_WRITE) != 0 && (prot & PROT_EXEC) != 0 {
        return Err("W^X violation: simultaneous PROT_WRITE and PROT_EXEC prohibited");
    }

    if addr < MMAP_START || addr >= MMAP_END {
        return Err("Address outside user mmap region");
    }

    let aligned_len = match length.checked_add(pmm::PAGE_SIZE - 1) {
        Some(l) => l & !(pmm::PAGE_SIZE - 1),
        None => return Err("Integer overflow in mprotect length"),
    };

    let target_end = match addr.checked_add(aligned_len) {
        Some(e) => e,
        None => return Err("Integer overflow in target mprotect range"),
    };

    let cur_pml4 = active_pml4();

    // Verify range resides inside an active VMA belonging to current address space
    let mut matching_vma = None;
    for i in 0..MAX_VMAS {
        let vma = VMA_TABLE[i];
        if vma.is_active && vma.pml4_phys == cur_pml4 && addr >= vma.start && target_end <= vma.end
        {
            matching_vma = Some(i);
            break;
        }
    }

    let vma_idx = matching_vma.ok_or("No matching active user VMA for mprotect range")?;
    let orig_vma = VMA_TABLE[vma_idx];

    // If protection is identical, no metadata update or splitting needed
    if orig_vma.prot == prot {
        return Ok(());
    }

    // Check required free slots before modifying page protections
    let is_exact = addr == orig_vma.start && target_end == orig_vma.end;
    let is_middle = addr > orig_vma.start && target_end < orig_vma.end;

    let required_slots = if is_exact {
        0
    } else if is_middle {
        2
    } else {
        1
    };

    let mut found_slots = [0usize; 2];
    let mut found_count = 0;
    if required_slots > 0 {
        for i in 0..MAX_VMAS {
            if !VMA_TABLE[i].is_active {
                found_slots[found_count] = i;
                found_count += 1;
                if found_count == required_slots {
                    break;
                }
            }
        }
        if found_count < required_slots {
            return Err("Max VMA capacity reached during partial mprotect split");
        }
    }

    let mut page_flags = 0u64;
    if (prot & PROT_WRITE) != 0 {
        page_flags |= PAGE_WRITABLE;
    }
    if (prot & PROT_EXEC) == 0 {
        page_flags |= PAGE_NO_EXECUTE;
    }

    let orig_page_flags = {
        let mut f = 0u64;
        if (orig_vma.prot & PROT_WRITE) != 0 {
            f |= PAGE_WRITABLE;
        }
        if (orig_vma.prot & PROT_EXEC) == 0 {
            f |= PAGE_NO_EXECUTE;
        }
        f
    };

    // Apply new protection flags across virtual pages with rollback on intermediate failure
    let mut offset = 0u64;
    while offset < aligned_len {
        let vaddr = addr + offset;
        if let Err(e) = mprotect_page(vaddr, page_flags) {
            let mut rollback_offset = 0u64;
            while rollback_offset < offset {
                let _ = mprotect_page(addr + rollback_offset, orig_page_flags);
                rollback_offset += pmm::PAGE_SIZE;
            }
            return Err(e);
        }
        offset += pmm::PAGE_SIZE;
    }

    // Split VMA metadata
    if is_exact {
        VMA_TABLE[vma_idx].prot = prot;
    } else if addr == orig_vma.start && target_end < orig_vma.end {
        // Front split: [addr, target_end) with new prot, [target_end, orig_vma.end) with old prot
        let new_slot = found_slots[0];
        VMA_TABLE[vma_idx].start = target_end;
        VMA_TABLE[new_slot] = Vma {
            pml4_phys: orig_vma.pml4_phys,
            start: addr,
            end: target_end,
            prot,
            flags: orig_vma.flags,
            is_active: true,
        };
    } else if addr > orig_vma.start && target_end == orig_vma.end {
        // Back split: [orig_vma.start, addr) with old prot, [addr, target_end) with new prot
        let new_slot = found_slots[0];
        VMA_TABLE[vma_idx].end = addr;
        VMA_TABLE[new_slot] = Vma {
            pml4_phys: orig_vma.pml4_phys,
            start: addr,
            end: target_end,
            prot,
            flags: orig_vma.flags,
            is_active: true,
        };
    } else if is_middle {
        // Middle split: [orig_vma.start, addr) old, [addr, target_end) new, [target_end, orig_vma.end) old
        let slot1 = found_slots[0];
        let slot2 = found_slots[1];
        VMA_TABLE[vma_idx].end = addr;
        VMA_TABLE[slot1] = Vma {
            pml4_phys: orig_vma.pml4_phys,
            start: addr,
            end: target_end,
            prot,
            flags: orig_vma.flags,
            is_active: true,
        };
        VMA_TABLE[slot2] = Vma {
            pml4_phys: orig_vma.pml4_phys,
            start: target_end,
            end: orig_vma.end,
            prot: orig_vma.prot,
            flags: orig_vma.flags,
            is_active: true,
        };
    }

    Ok(())
}

/// Alias for sys_mmap with anonymous mapping flags.
pub unsafe fn mmap_anonymous(
    hint_addr: u64,
    length: u64,
    prot: u32,
    flags: u32,
) -> Result<u64, &'static str> {
    sys_mmap(hint_addr, length, prot, flags | MAP_ANONYMOUS)
}

/// Alias for sys_munmap.
pub unsafe fn munmap_pages(addr: u64, length: u64) -> Result<(), &'static str> {
    sys_munmap(addr, length)
}

/// Alias for sys_mprotect.
pub unsafe fn mprotect_pages(addr: u64, length: u64, prot: u32) -> Result<(), &'static str> {
    sys_mprotect(addr, length, prot)
}

/// Explicit failure for madvise stub.
pub unsafe fn madvise_pages(_addr: u64, _length: u64, _advice: u32) -> Result<(), &'static str> {
    Err("madvise is not implemented (ENOSYS)")
}

/// Validate that a virtual address range does not overflow and stays within user space.
pub fn validate_virt_addr_range(addr: u64, length: u64) -> Result<(), &'static str> {
    if length == 0 {
        return Ok(());
    }
    let end = match addr.checked_add(length) {
        Some(e) => e,
        None => return Err("Integer overflow in address calculation"),
    };
    if addr >= 0x10000 && end <= 0x0000_7FFF_FFFF_FFFF {
        Ok(())
    } else {
        Err("Address range resides outside user space boundaries")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vma_range_validation() {
        assert!(validate_virt_addr_range(0x40000000, 0x1000).is_ok());
        assert!(validate_virt_addr_range(0x5000_0000_0000, 0x2000).is_ok());
        assert!(validate_virt_addr_range(0, 0x1000).is_err());
        assert!(validate_virt_addr_range(0x0000_8000_0000_0000, 0x1000).is_err());
    }

    #[test]
    fn test_find_free_mmap_range() {
        unsafe {
            cleanup_vmas_for_pml4(0x1000);
            let free_addr = find_free_mmap_range(0x1000, 0x2000);
            assert_eq!(free_addr, Some(MMAP_START));
        }
    }

    #[test]
    fn test_wx_violation_rejection() {
        unsafe {
            let res = sys_mmap(0, 0x1000, PROT_WRITE | PROT_EXEC, MAP_ANONYMOUS);
            assert!(res.is_err());
            assert_eq!(
                res.unwrap_err(),
                "W^X violation: simultaneous PROT_WRITE and PROT_EXEC prohibited"
            );
        }
    }

    #[test]
    fn test_mmap_arg_validation() {
        unsafe {
            assert!(sys_mmap(0, 0, PROT_READ, MAP_ANONYMOUS).is_err());
            assert!(sys_mmap(0, 0x1000, 0xFF, MAP_ANONYMOUS).is_err());
        }
    }
}
