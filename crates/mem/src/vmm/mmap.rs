// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! User virtual memory area (VMA) allocator, mmap, munmap, and mprotect operations.

use super::paging::{
    free_and_unmap_page, map_page, mprotect_page, PAGE_NO_EXECUTE, PAGE_PRESENT, PAGE_USER,
    PAGE_WRITABLE,
};
use crate::pmm;

pub const PROT_NONE: u32 = 0;
pub const PROT_READ: u32 = 1 << 0;
pub const PROT_WRITE: u32 = 1 << 1;
pub const PROT_EXEC: u32 = 1 << 2;

pub const MAP_SHARED: u32 = 1 << 0;
pub const MAP_PRIVATE: u32 = 1 << 1;
pub const MAP_FIXED: u32 = 1 << 4;
pub const MAP_ANONYMOUS: u32 = 1 << 5;

pub const MMAP_START: u64 = 0x5000_0000_0000;
pub const MMAP_END: u64 = 0x7000_0000_0000;

static mut NEXT_MMAP_ADDR: u64 = MMAP_START;

/// Allocate and map an anonymous virtual memory region for user space.
pub unsafe fn sys_mmap(
    hint_addr: u64,
    length: u64,
    prot: u32,
    flags: u32,
) -> Result<u64, &'static str> {
    if length == 0 {
        return Err("Invalid mmap length (0)");
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

    Ok(start_vaddr)
}

/// Unmap a user virtual memory region and release underlying physical frames.
pub unsafe fn sys_munmap(addr: u64, length: u64) -> Result<(), &'static str> {
    if length == 0 || !addr.is_multiple_of(pmm::PAGE_SIZE) {
        return Err("Invalid address alignment or zero length for munmap");
    }

    let aligned_len = match length.checked_add(pmm::PAGE_SIZE - 1) {
        Some(l) => l & !(pmm::PAGE_SIZE - 1),
        None => return Err("Integer overflow in munmap length"),
    };

    let mut offset = 0u64;
    while offset < aligned_len {
        let vaddr = addr + offset;
        let _ = free_and_unmap_page(vaddr);
        offset += pmm::PAGE_SIZE;
    }

    Ok(())
}

/// Change protection flags on an existing mapped user memory range.
pub unsafe fn sys_mprotect(addr: u64, length: u64, prot: u32) -> Result<(), &'static str> {
    if length == 0 || !addr.is_multiple_of(pmm::PAGE_SIZE) {
        return Err("Invalid address alignment or zero length for mprotect");
    }

    let aligned_len = match length.checked_add(pmm::PAGE_SIZE - 1) {
        Some(l) => l & !(pmm::PAGE_SIZE - 1),
        None => return Err("Integer overflow in mprotect length"),
    };

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

/// Stub/advisory for madvise.
pub unsafe fn madvise_pages(_addr: u64, _length: u64, _advice: u32) -> Result<(), &'static str> {
    Ok(())
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
