// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! User virtual memory area (VMA) allocator, authoritative VMA tracking, mmap, munmap, and mprotect.

use super::paging::{
    free_and_unmap_page, map_page, mprotect_page, PAGE_NO_EXECUTE, PAGE_PRESENT, PAGE_USER,
    PAGE_WRITABLE,
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

#[derive(Clone, Copy)]
pub struct Vma {
    pub start: u64,
    pub end: u64,
    pub prot: u32,
    pub flags: u32,
    pub is_active: bool,
}

pub const MAX_VMAS: usize = 32;
static mut VMA_TABLE: [Vma; MAX_VMAS] = [Vma {
    start: 0,
    end: 0,
    prot: 0,
    flags: 0,
    is_active: false,
}; MAX_VMAS];

static mut NEXT_MMAP_ADDR: u64 = MMAP_START;

/// Allocate and map an anonymous virtual memory region for user space with VMA bookkeeping.
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

    let aligned_len = match length.checked_add(pmm::PAGE_SIZE - 1) {
        Some(l) => l & !(pmm::PAGE_SIZE - 1),
        None => return Err("Integer overflow calculating mmap length"),
    };

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

        // Check for collision with existing active VMAs
        for i in 0..MAX_VMAS {
            let vma = VMA_TABLE[i];
            if vma.is_active && !(fixed_end <= vma.start || hint_addr >= vma.end) {
                return Err("MAP_FIXED collision with existing VMA mapping");
            }
        }
        hint_addr
    } else {
        let vaddr = NEXT_MMAP_ADDR;
        let next = match vaddr.checked_add(aligned_len) {
            Some(n) => n,
            None => return Err("Integer overflow allocating mmap virtual address"),
        };
        if next > MMAP_END {
            return Err("Out of virtual address space in mmap region");
        }
        NEXT_MMAP_ADDR = next;
        vaddr
    };

    // Find free VMA slot
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

    // Record authoritative VMA
    VMA_TABLE[slot_idx] = Vma {
        start: start_vaddr,
        end: start_vaddr + aligned_len,
        prot,
        flags,
        is_active: true,
    };

    Ok(start_vaddr)
}

/// Unmap a user virtual memory region and release underlying physical frames after VMA ownership validation.
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

    // Verify range resides inside an active VMA
    let mut matching_vma = None;
    for i in 0..MAX_VMAS {
        let vma = VMA_TABLE[i];
        if vma.is_active && addr >= vma.start && target_end <= vma.end {
            matching_vma = Some(i);
            break;
        }
    }

    let vma_idx = matching_vma.ok_or("No matching active user VMA for munmap range")?;

    let mut offset = 0u64;
    while offset < aligned_len {
        let vaddr = addr + offset;
        let _ = free_and_unmap_page(vaddr);
        offset += pmm::PAGE_SIZE;
    }

    // Clean up VMA entry
    if addr == VMA_TABLE[vma_idx].start && target_end == VMA_TABLE[vma_idx].end {
        VMA_TABLE[vma_idx].is_active = false;
    }

    Ok(())
}

/// Change protection flags on an existing mapped user memory range after VMA validation.
pub unsafe fn sys_mprotect(addr: u64, length: u64, prot: u32) -> Result<(), &'static str> {
    if length == 0 || !addr.is_multiple_of(pmm::PAGE_SIZE) {
        return Err("Invalid address alignment or zero length for mprotect");
    }

    if (prot & !SUPPORTED_PROT) != 0 {
        return Err("Invalid protection flags (EINVAL)");
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

    // Verify range resides inside an active VMA
    let mut matching_vma = None;
    for i in 0..MAX_VMAS {
        let vma = VMA_TABLE[i];
        if vma.is_active && addr >= vma.start && target_end <= vma.end {
            matching_vma = Some(i);
            break;
        }
    }

    let vma_idx = matching_vma.ok_or("No matching active user VMA for mprotect range")?;

    let mut page_flags = 0u64;
    if (prot & PROT_WRITE) != 0 {
        page_flags |= PAGE_WRITABLE;
    }
    if (prot & PROT_EXEC) == 0 {
        page_flags |= PAGE_NO_EXECUTE;
    }

    let mut offset = 0u64;
    while offset < aligned_len {
        let vaddr = addr + offset;
        mprotect_page(vaddr, page_flags)?;
        offset += pmm::PAGE_SIZE;
    }

    VMA_TABLE[vma_idx].prot = prot;
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
