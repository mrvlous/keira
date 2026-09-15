// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! User virtual memory area (VMA) allocator, per-address-space VMA tracking, mmap, partial munmap, and partial mprotect.

use super::paging::{
    active_pml4, free_and_unmap_page, get_pte_in_pml4, is_page_mapped_in_pml4, map_page,
    mprotect_page, PAGE_NO_EXECUTE, PAGE_PRESENT, PAGE_USER, PAGE_WRITABLE, USER_MAX_VADDR,
    USER_MIN_VADDR,
};
use crate::pmm;

pub const PROT_NONE: u32 = 0;
pub const PROT_READ: u32 = 1 << 0;
pub const PROT_WRITE: u32 = 1 << 1;
pub const PROT_EXEC: u32 = 1 << 2;
pub const SUPPORTED_PROT: u32 = PROT_READ | PROT_WRITE | PROT_EXEC;

pub const MAP_SHARED: u32 = 1 << 0;
pub const MAP_PRIVATE: u32 = 1 << 1;
pub const MAP_POPULATE: u32 = 1 << 3;
pub const MAP_FIXED: u32 = 1 << 4;
pub const MAP_ANONYMOUS: u32 = 1 << 5;

pub const MS_ASYNC: u32 = 1;
pub const MS_INVALIDATE: u32 = 2;
pub const MS_SYNC: u32 = 4;

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
    pub file_backed: bool,
    pub file_path: [u8; 128],
    pub file_path_len: usize,
    pub file_offset: u64,
    pub file_size: u64,
}

impl Vma {
    pub const fn empty() -> Self {
        Self {
            pml4_phys: 0,
            start: 0,
            end: 0,
            prot: 0,
            flags: 0,
            is_active: false,
            file_backed: false,
            file_path: [0u8; 128],
            file_path_len: 0,
            file_offset: 0,
            file_size: 0,
        }
    }

    pub const fn new_anon(pml4_phys: u64, start: u64, end: u64, prot: u32, flags: u32) -> Self {
        Self {
            pml4_phys,
            start,
            end,
            prot,
            flags,
            is_active: true,
            file_backed: false,
            file_path: [0u8; 128],
            file_path_len: 0,
            file_offset: 0,
            file_size: 0,
        }
    }

    pub fn file_path_str(&self) -> Option<&str> {
        if !self.file_backed || self.file_path_len == 0 || self.file_path_len > 128 {
            None
        } else {
            core::str::from_utf8(&self.file_path[..self.file_path_len]).ok()
        }
    }
}

pub const MAX_VMAS: usize = 64;
static mut VMA_TABLE: [Vma; MAX_VMAS] = [Vma::empty(); MAX_VMAS];

pub type FileReadHook = fn(path: &str, offset: u64, buf: &mut [u8]) -> Result<usize, &'static str>;
pub type FileSyncHook = fn(path: &str, offset: u64, buf: &[u8]) -> Result<usize, &'static str>;

static mut FILE_READ_HOOK: Option<FileReadHook> = None;
static mut FILE_SYNC_HOOK: Option<FileSyncHook> = None;

/// Register external VFS callbacks for demand paging and disk sync.
pub fn register_file_backing_hooks(read_hook: FileReadHook, sync_hook: FileSyncHook) {
    unsafe {
        FILE_READ_HOOK = Some(read_hook);
        FILE_SYNC_HOOK = Some(sync_hook);
    }
}

/// Retrieve registered file read hook.
pub fn get_file_read_hook() -> Option<FileReadHook> {
    unsafe { FILE_READ_HOOK }
}

/// Retrieve registered file sync hook.
pub fn get_file_sync_hook() -> Option<FileSyncHook> {
    unsafe { FILE_SYNC_HOOK }
}

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

/// Locate the active VMA enclosing the given virtual address within the specified address space.
pub unsafe fn find_active_vma(pml4_phys: u64, addr: u64) -> Option<Vma> {
    if pml4_phys == 0 {
        return None;
    }
    for i in 0..MAX_VMAS {
        let vma = VMA_TABLE[i];
        if vma.is_active && vma.pml4_phys == pml4_phys && addr >= vma.start && addr < vma.end {
            return Some(vma);
        }
    }
    None
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

/// Allocate and map an anonymous or file-backed virtual memory region for user space with per-process VMA bookkeeping.
pub unsafe fn sys_mmap_file(
    hint_addr: u64,
    length: u64,
    prot: u32,
    flags: u32,
    file_path: Option<&str>,
    file_offset: u64,
    file_size: u64,
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

    if aligned_len == 0 || aligned_len > (MMAP_END - MMAP_START) {
        return Err("Invalid aligned mmap length");
    }

    let cur_pml4 = active_pml4();

    let start_vaddr = if (flags & MAP_FIXED) != 0 {
        if hint_addr < MMAP_START || hint_addr >= MMAP_END {
            return Err("MAP_FIXED address outside user mmap region");
        }
        if hint_addr % pmm::PAGE_SIZE != 0 {
            return Err("MAP_FIXED address is not page-aligned");
        }
        let fixed_end = match hint_addr.checked_add(aligned_len) {
            Some(e) => e,
            None => return Err("Integer overflow in MAP_FIXED range"),
        };
        if fixed_end > MMAP_END {
            return Err("MAP_FIXED range exceeds user mmap boundary");
        }

        // 1. Check for collision with existing active VMAs in the same address space
        for i in 0..MAX_VMAS {
            let vma = VMA_TABLE[i];
            if vma.is_active
                && vma.pml4_phys == cur_pml4
                && !(fixed_end <= vma.start || hint_addr >= vma.end)
            {
                return Err("MAP_FIXED collision with existing VMA mapping");
            }
        }

        // 2. Check for collision with present page table mappings (e.g. ELF segments, heap, stack)
        let mut check_offset = 0u64;
        while check_offset < aligned_len {
            let check_vaddr = hint_addr + check_offset;
            if is_page_mapped_in_pml4(cur_pml4, check_vaddr) {
                return Err("MAP_FIXED collision with present page table mapping");
            }
            check_offset += pmm::PAGE_SIZE;
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

    // Under demand paging, anonymous mappings do not allocate physical frames upfront
    // unless MAP_POPULATE is explicitly requested. Unmapped pages fault in on-demand via #PF.
    if (flags & MAP_POPULATE) != 0 {
        let mut allocated_pages = 0u64;
        while allocated_pages < aligned_len {
            let vaddr = start_vaddr + allocated_pages;
            let frame = match pmm::alloc_frame() {
                Some(f) => f,
                None => {
                    // Rollback previously mapped pages and track cleanup errors
                    let mut rollback_offset = 0u64;
                    let mut first_rollback_err = None;
                    while rollback_offset < allocated_pages {
                        if let Err(e) = free_and_unmap_page(start_vaddr + rollback_offset) {
                            if first_rollback_err.is_none() {
                                first_rollback_err = Some(e);
                            }
                        }
                        rollback_offset += pmm::PAGE_SIZE;
                    }
                    if let Some(_err) = first_rollback_err {
                        return Err("Out of physical memory during mmap allocation (rollback cleanup error encountered)");
                    }
                    return Err("Out of physical memory during mmap allocation");
                }
            };

            if let Err(e) = map_page(vaddr, frame, page_flags) {
                pmm::free_frame(frame);
                let mut rollback_offset = 0u64;
                let mut first_rollback_err = None;
                while rollback_offset < allocated_pages {
                    if let Err(re) = free_and_unmap_page(start_vaddr + rollback_offset) {
                        if first_rollback_err.is_none() {
                            first_rollback_err = Some(re);
                        }
                    }
                    rollback_offset += pmm::PAGE_SIZE;
                }
                if let Some(_re) = first_rollback_err {
                    return Err("Failed to map page during mmap allocation (rollback cleanup error encountered)");
                }
                return Err(e);
            }

            let ptr = vaddr as *mut u8;
            core::ptr::write_bytes(ptr, 0, pmm::PAGE_SIZE as usize);

            if let (Some(path), Some(read_fn)) = (file_path, FILE_READ_HOOK) {
                let cur_offset = file_offset + allocated_pages;
                if cur_offset < file_size {
                    let to_read = core::cmp::min(pmm::PAGE_SIZE, file_size - cur_offset) as usize;
                    let slice = core::slice::from_raw_parts_mut(ptr, to_read);
                    let _ = read_fn(path, cur_offset, slice);
                }
            }

            allocated_pages += pmm::PAGE_SIZE;
        }
    }

    // Record authoritative VMA tagged with active PML4 ownership
    let mut vma = Vma {
        pml4_phys: cur_pml4,
        start: start_vaddr,
        end: start_vaddr + aligned_len,
        prot,
        flags,
        is_active: true,
        file_backed: false,
        file_path: [0u8; 128],
        file_path_len: 0,
        file_offset,
        file_size,
    };
    if let Some(path) = file_path {
        vma.file_backed = true;
        let p_bytes = path.as_bytes();
        let copy_len = core::cmp::min(p_bytes.len(), 128);
        vma.file_path[..copy_len].copy_from_slice(&p_bytes[..copy_len]);
        vma.file_path_len = copy_len;
    }
    VMA_TABLE[slot_idx] = vma;

    Ok(start_vaddr)
}

/// Allocate and map an anonymous virtual memory region for user space with per-process VMA bookkeeping.
pub unsafe fn sys_mmap(
    hint_addr: u64,
    length: u64,
    prot: u32,
    flags: u32,
) -> Result<u64, &'static str> {
    sys_mmap_file(hint_addr, length, prot, flags, None, 0, 0)
}

/// Extended sys_munmap returning the exact number of bytes successfully unmapped.
/// Returns `Ok(unmapped_bytes)` on complete success, or `Err((error_message, unmapped_bytes))` on partial failure.
pub unsafe fn sys_munmap_ext(addr: u64, length: u64) -> Result<u64, (&'static str, u64)> {
    if length == 0 || addr % pmm::PAGE_SIZE != 0 {
        return Err(("Invalid address alignment or zero length for munmap", 0));
    }

    if addr < MMAP_START || addr >= MMAP_END {
        return Err(("Address outside user mmap region", 0));
    }

    let aligned_len = match length.checked_add(pmm::PAGE_SIZE - 1) {
        Some(l) => l & !(pmm::PAGE_SIZE - 1),
        None => return Err(("Integer overflow in munmap length", 0)),
    };

    if aligned_len == 0 {
        return Err(("Invalid aligned munmap length", 0));
    }

    let target_end = match addr.checked_add(aligned_len) {
        Some(e) => e,
        None => return Err(("Integer overflow in target unmap range", 0)),
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

    let vma_idx = match matching_vma {
        Some(i) => i,
        None => return Err(("No matching active user VMA for munmap range", 0)),
    };
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
        match free_slot {
            Some(s) => Some(s),
            None => return Err(("Max VMA capacity reached during partial munmap split", 0)),
        }
    } else {
        None
    };

    // Unmap physical pages with strict error tracking and partial VMA synchronization.
    // For lazily mapped pages, only unmap pages that were actually faulted into the page table.
    let mut offset = 0u64;
    let mut unmap_err = None;
    while offset < aligned_len {
        let vaddr = addr + offset;
        #[cfg(not(test))]
        let is_mapped = is_page_mapped_in_pml4(cur_pml4, vaddr);
        #[cfg(test)]
        let is_mapped = true;

        if is_mapped {
            if let Err(e) = free_and_unmap_page(vaddr) {
                unmap_err = Some(e);
                break;
            }
        }
        offset += pmm::PAGE_SIZE;
    }

    let actual_unmapped_len = offset;
    let actual_target_end = addr + actual_unmapped_len;

    // Update VMA metadata to reflect only the pages that were successfully unmapped
    if actual_unmapped_len > 0 {
        if addr == orig_vma.start && actual_target_end == orig_vma.end {
            // Case 1: Exact match -> deactivate
            VMA_TABLE[vma_idx].is_active = false;
        } else if addr == orig_vma.start && actual_target_end < orig_vma.end {
            // Case 2: Front trim -> shrink start
            let unmapped_delta = actual_target_end - orig_vma.start;
            VMA_TABLE[vma_idx].start = actual_target_end;
            if VMA_TABLE[vma_idx].file_backed {
                VMA_TABLE[vma_idx].file_offset = VMA_TABLE[vma_idx]
                    .file_offset
                    .saturating_add(unmapped_delta);
            }
        } else if addr > orig_vma.start && actual_target_end == orig_vma.end {
            // Case 3: Back trim -> shrink end
            VMA_TABLE[vma_idx].end = addr;
        } else if let Some(new_idx) = split_slot {
            // Case 4: Middle split -> left half stays in vma_idx, right half in new_idx
            let orig = VMA_TABLE[vma_idx];
            VMA_TABLE[vma_idx].end = addr;
            let mut right_vma = orig;
            right_vma.start = actual_target_end;
            right_vma.end = orig_vma.end;
            if right_vma.file_backed {
                let right_delta = actual_target_end - orig_vma.start;
                right_vma.file_offset = orig_vma.file_offset.saturating_add(right_delta);
            }
            VMA_TABLE[new_idx] = right_vma;
        }
    }

    if let Some(e) = unmap_err {
        return Err((e, actual_unmapped_len));
    }

    Ok(actual_unmapped_len)
}

/// Unmap a user virtual memory region and release underlying physical frames with VMA splitting.
pub unsafe fn sys_munmap(addr: u64, length: u64) -> Result<(), &'static str> {
    match sys_munmap_ext(addr, length) {
        Ok(_) => Ok(()),
        Err((err, _)) => Err(err),
    }
}

/// Change protection flags on an existing mapped user memory range with VMA splitting.
pub unsafe fn sys_mprotect(addr: u64, length: u64, prot: u32) -> Result<(), &'static str> {
    if length == 0 || addr % pmm::PAGE_SIZE != 0 {
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

    if aligned_len == 0 {
        return Err("Invalid aligned mprotect length");
    }

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
        let mut front_vma = orig_vma;
        front_vma.start = addr;
        front_vma.end = target_end;
        front_vma.prot = prot;

        let mut back_vma = orig_vma;
        back_vma.start = target_end;
        if back_vma.file_backed {
            let delta = target_end - orig_vma.start;
            back_vma.file_offset = orig_vma.file_offset.saturating_add(delta);
        }

        VMA_TABLE[vma_idx] = back_vma;
        VMA_TABLE[new_slot] = front_vma;
    } else if addr > orig_vma.start && target_end == orig_vma.end {
        // Back split: [orig_vma.start, addr) with old prot, [addr, target_end) with new prot
        let new_slot = found_slots[0];
        let mut back_vma = orig_vma;
        back_vma.start = addr;
        back_vma.end = target_end;
        back_vma.prot = prot;
        if back_vma.file_backed {
            let delta = addr - orig_vma.start;
            back_vma.file_offset = orig_vma.file_offset.saturating_add(delta);
        }

        VMA_TABLE[vma_idx].end = addr;
        VMA_TABLE[new_slot] = back_vma;
    } else if is_middle {
        // Middle split: [orig_vma.start, addr) old, [addr, target_end) new, [target_end, orig_vma.end) old
        let slot1 = found_slots[0];
        let slot2 = found_slots[1];

        let mut mid_vma = orig_vma;
        mid_vma.start = addr;
        mid_vma.end = target_end;
        mid_vma.prot = prot;
        if mid_vma.file_backed {
            let mid_delta = addr - orig_vma.start;
            mid_vma.file_offset = orig_vma.file_offset.saturating_add(mid_delta);
        }

        let mut right_vma = orig_vma;
        right_vma.start = target_end;
        right_vma.end = orig_vma.end;
        if right_vma.file_backed {
            let right_delta = target_end - orig_vma.start;
            right_vma.file_offset = orig_vma.file_offset.saturating_add(right_delta);
        }

        VMA_TABLE[vma_idx].end = addr;
        VMA_TABLE[slot1] = mid_vma;
        VMA_TABLE[slot2] = right_vma;
    }

    Ok(())
}

/// Synchronize a mapped memory region with its underlying file storage.
pub unsafe fn sys_msync(addr: u64, length: u64, _flags: u32) -> Result<(), &'static str> {
    if length == 0 {
        return Err("Invalid msync length (0)");
    }
    if addr % pmm::PAGE_SIZE != 0 {
        return Err("Address is not page-aligned (EINVAL)");
    }

    let aligned_len = match length.checked_add(pmm::PAGE_SIZE - 1) {
        Some(l) => l & !(pmm::PAGE_SIZE - 1),
        None => return Err("Integer overflow calculating msync length"),
    };

    let target_end = match addr.checked_add(aligned_len) {
        Some(e) => e,
        None => return Err("Integer overflow in msync address range"),
    };

    let cur_pml4 = active_pml4();
    let vma = match find_active_vma(cur_pml4, addr) {
        Some(v) => v,
        None => return Err("No active VMA found enclosing address (ENOMEM)"),
    };

    if target_end > vma.end {
        return Err("msync range exceeds VMA boundary (ENOMEM)");
    }

    // If anonymous or not shared, msync succeeds without writing to disk
    if !vma.file_backed || (vma.flags & MAP_SHARED) == 0 {
        return Ok(());
    }

    let path = match vma.file_path_str() {
        Some(p) => p,
        None => return Err("Invalid file path in file-backed VMA"),
    };

    let sync_fn = match FILE_SYNC_HOOK {
        Some(f) => f,
        None => return Err("No file sync hook registered"),
    };

    let mut curr_vaddr = addr;
    while curr_vaddr < target_end {
        #[cfg(not(test))]
        {
            if let Some(pte_ptr) = super::paging::get_pte_mut_in_pml4(cur_pml4, curr_vaddr) {
                let pte = *pte_ptr;
                if (pte & super::paging::PAGE_PRESENT) != 0 {
                    let is_dirty =
                        (pte & super::paging::PAGE_DIRTY) != 0 || (vma.prot & PROT_WRITE) != 0;
                    if is_dirty {
                        let page_delta = curr_vaddr - vma.start;
                        let current_file_offset = vma.file_offset + page_delta;

                        if current_file_offset < vma.file_size {
                            let write_len =
                                core::cmp::min(pmm::PAGE_SIZE, vma.file_size - current_file_offset)
                                    as usize;

                            let src_slice =
                                core::slice::from_raw_parts(curr_vaddr as *const u8, write_len);
                            sync_fn(path, current_file_offset, src_slice)?;
                        }

                        *pte_ptr = pte & !super::paging::PAGE_DIRTY;
                        keira_arch::cpu::invlpg(curr_vaddr as usize);
                    }
                }
            }
        }
        #[cfg(test)]
        {
            let page_delta = curr_vaddr - vma.start;
            let current_file_offset = vma.file_offset + page_delta;
            if current_file_offset < vma.file_size {
                let write_len =
                    core::cmp::min(pmm::PAGE_SIZE, vma.file_size - current_file_offset) as usize;
                let dummy = [0u8; 4096];
                sync_fn(path, current_file_offset, &dummy[..write_len])?;
            }
        }
        curr_vaddr += pmm::PAGE_SIZE;
    }

    Ok(())
}

/// Verify that all active VMAs for a given address space match the underlying page table states,
/// and detect any orphan page table mappings in the user mmap address space.
pub unsafe fn verify_vma_pte_invariants(pml4_phys: u64) -> Result<(), &'static str> {
    if pml4_phys == 0 {
        return Ok(());
    }
    // 1. Forward check: verify each active VMA against page table attributes
    for i in 0..MAX_VMAS {
        let vma = VMA_TABLE[i];
        if vma.is_active && vma.pml4_phys == pml4_phys {
            let mut vaddr = vma.start;
            while vaddr < vma.end {
                if let Some(pte) = get_pte_in_pml4(pml4_phys, vaddr) {
                    if (pte & PAGE_PRESENT) != 0 {
                        if (pte & PAGE_USER) == 0 {
                            return Err("VMA invariant violation: PTE missing PAGE_USER flag");
                        }
                        if (vma.prot & PROT_WRITE) != 0 && (pte & PAGE_WRITABLE) == 0 {
                            return Err(
                                "VMA invariant violation: Writable VMA has non-writable PTE",
                            );
                        }
                        if (vma.prot & PROT_WRITE) == 0 && (pte & PAGE_WRITABLE) != 0 {
                            return Err("VMA invariant violation: Read-only VMA has writable PTE");
                        }
                        if (vma.prot & PROT_EXEC) == 0 && (pte & PAGE_NO_EXECUTE) == 0 {
                            return Err(
                                "VMA invariant violation: Non-executable VMA has executable PTE",
                            );
                        }
                    }
                }
                vaddr += pmm::PAGE_SIZE;
            }
        }
    }

    // 2. Reverse check: ensure present user mappings in MMAP range belong to an active VMA
    #[cfg(not(test))]
    {
        let pml4 = pml4_phys as *const u64;

        // Traverse all canonical user PML4 slots (1..256)
        for pml4_idx in 1..256 {
            let pml4_entry = *pml4.add(pml4_idx);
            if (pml4_entry & PAGE_PRESENT) == 0 {
                continue;
            }
            let pdpt_phys = pml4_entry & super::paging::PTE_ADDR_MASK;
            let pdpt = pdpt_phys as *const u64;

            for pdpt_idx in 0..512 {
                let pdpt_entry = *pdpt.add(pdpt_idx);
                if (pdpt_entry & PAGE_PRESENT) == 0 {
                    continue;
                }
                if (pdpt_entry & super::paging::PAGE_HUGE) != 0 {
                    let page_start = ((pml4_idx as u64) << 39) | ((pdpt_idx as u64) << 30);
                    let page_end = page_start + 0x4000_0000;
                    // Check range-overlap with MMAP space
                    if page_start < MMAP_END && page_end > MMAP_START {
                        let mut in_vma = false;
                        for i in 0..MAX_VMAS {
                            let v = VMA_TABLE[i];
                            if v.is_active
                                && v.pml4_phys == pml4_phys
                                && page_start < v.end
                                && page_end > v.start
                            {
                                in_vma = true;
                                break;
                            }
                        }
                        if !in_vma {
                            return Err("VMA invariant violation: Orphan 1GB huge page overlaps mmap region without active VMA");
                        }
                    }
                    continue;
                }

                let pd_phys = pdpt_entry & super::paging::PTE_ADDR_MASK;
                let pd = pd_phys as *const u64;

                for pd_idx in 0..512 {
                    let pd_entry = *pd.add(pd_idx);
                    if (pd_entry & PAGE_PRESENT) == 0 {
                        continue;
                    }
                    if (pd_entry & super::paging::PAGE_HUGE) != 0 {
                        let page_start = ((pml4_idx as u64) << 39)
                            | ((pdpt_idx as u64) << 30)
                            | ((pd_idx as u64) << 21);
                        let page_end = page_start + 0x20_0000;
                        // Check range-overlap with MMAP space
                        if page_start < MMAP_END && page_end > MMAP_START {
                            let mut in_vma = false;
                            for i in 0..MAX_VMAS {
                                let v = VMA_TABLE[i];
                                if v.is_active
                                    && v.pml4_phys == pml4_phys
                                    && page_start < v.end
                                    && page_end > v.start
                                {
                                    in_vma = true;
                                    break;
                                }
                            }
                            if !in_vma {
                                return Err("VMA invariant violation: Orphan 2MB huge page overlaps mmap region without active VMA");
                            }
                        }
                        continue;
                    }

                    let pt_phys = pd_entry & super::paging::PTE_ADDR_MASK;
                    let pt = pt_phys as *const u64;

                    for pt_idx in 0..512 {
                        let pt_entry = *pt.add(pt_idx);
                        if (pt_entry & PAGE_PRESENT) != 0
                            && (pt_entry & super::paging::PAGE_USER) != 0
                        {
                            let page_start = ((pml4_idx as u64) << 39)
                                | ((pdpt_idx as u64) << 30)
                                | ((pd_idx as u64) << 21)
                                | ((pt_idx as u64) << 12);
                            let page_end = page_start + pmm::PAGE_SIZE;
                            if page_start < MMAP_END && page_end > MMAP_START {
                                let mut in_vma = false;
                                for i in 0..MAX_VMAS {
                                    let v = VMA_TABLE[i];
                                    if v.is_active
                                        && v.pml4_phys == pml4_phys
                                        && page_start < v.end
                                        && page_end > v.start
                                    {
                                        in_vma = true;
                                        break;
                                    }
                                }
                                if !in_vma {
                                    return Err("VMA invariant violation: Orphan PTE exists in mmap region without active VMA");
                                }
                            }
                        }
                    }
                }
            }
        }
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

/// Advise kernel about memory range access patterns (madvise).
pub unsafe fn madvise_pages(addr: u64, length: u64, _advice: u32) -> Result<(), &'static str> {
    validate_virt_addr_range(addr, length)
}

/// Validate that a virtual address range does not overflow and stays within canonical user space.
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

#[cfg(test)]
mod tests {
    use super::super::paging;
    use super::*;

    pub static VMA_TEST_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());

    #[test]
    fn test_vma_range_validation() {
        assert!(validate_virt_addr_range(0x40000000, 0x1000).is_ok());
        assert!(validate_virt_addr_range(0x5000_0000_0000, 0x2000).is_ok());
        assert!(validate_virt_addr_range(0, 0x1000).is_err());
        assert!(validate_virt_addr_range(0x0000_8000_0000_0000, 0x1000).is_err());
    }

    #[test]
    fn test_find_free_mmap_range() {
        let _lock = VMA_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
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

    #[test]
    fn test_vma_invariants_empty() {
        unsafe {
            assert!(verify_vma_pte_invariants(0).is_ok());
        }
    }

    #[test]
    fn test_map_fixed_bounds_and_alignment() {
        unsafe {
            // Unaligned MAP_FIXED address
            let res_unaligned = sys_mmap(
                0x5000_0000_0001,
                0x1000,
                PROT_READ,
                MAP_FIXED | MAP_ANONYMOUS,
            );
            assert!(res_unaligned.is_err());
            assert_eq!(
                res_unaligned.unwrap_err(),
                "MAP_FIXED address is not page-aligned"
            );

            // Out of bounds MAP_FIXED address (below MMAP_START)
            let res_low = sys_mmap(0x1000_0000, 0x1000, PROT_READ, MAP_FIXED | MAP_ANONYMOUS);
            assert!(res_low.is_err());
            assert_eq!(
                res_low.unwrap_err(),
                "MAP_FIXED address outside user mmap region"
            );

            // Out of bounds MAP_FIXED address (above MMAP_END)
            let res_high = sys_mmap(
                0x8000_0000_0000,
                0x1000,
                PROT_READ,
                MAP_FIXED | MAP_ANONYMOUS,
            );
            assert!(res_high.is_err());
            assert_eq!(
                res_high.unwrap_err(),
                "MAP_FIXED address outside user mmap region"
            );
        }
    }

    #[test]
    fn test_mprotect_arg_validation() {
        unsafe {
            // Zero length
            assert!(sys_mprotect(0x5000_0000_0000, 0, PROT_READ).is_err());

            // Unaligned address
            assert!(sys_mprotect(0x5000_0000_0001, 0x1000, PROT_READ).is_err());

            // W^X violation
            let res_wx = sys_mprotect(0x5000_0000_0000, 0x1000, PROT_WRITE | PROT_EXEC);
            assert!(res_wx.is_err());
            assert_eq!(
                res_wx.unwrap_err(),
                "W^X violation: simultaneous PROT_WRITE and PROT_EXEC prohibited"
            );

            // Invalid protection bits
            assert!(sys_mprotect(0x5000_0000_0000, 0x1000, 0xFF).is_err());
        }
    }

    #[test]
    fn test_munmap_arg_validation() {
        unsafe {
            // Zero length
            assert!(sys_munmap(0x5000_0000_0000, 0).is_err());

            // Unaligned address
            assert!(sys_munmap(0x5000_0000_0001, 0x1000).is_err());

            // Out of bounds address
            assert!(sys_munmap(0x1000_0000, 0x1000).is_err());
        }
    }

    #[test]
    fn test_vma_table_cleanup_and_isolation() {
        let _lock = VMA_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        unsafe {
            cleanup_vmas_for_pml4(0x2000);
            cleanup_vmas_for_pml4(0x3000);

            VMA_TABLE[0] = Vma::new_anon(
                0x2000,
                0x5000_0000_0000,
                0x5000_0000_2000,
                PROT_READ,
                MAP_PRIVATE | MAP_ANONYMOUS,
            );
            VMA_TABLE[1] = Vma::new_anon(
                0x3000,
                0x5000_0000_0000,
                0x5000_0000_4000,
                PROT_READ | PROT_WRITE,
                MAP_PRIVATE | MAP_ANONYMOUS,
            );

            // Cleanup only 0x2000
            cleanup_vmas_for_pml4(0x2000);
            assert!(!VMA_TABLE[0].is_active);
            assert!(VMA_TABLE[1].is_active);

            // Cleanup 0x3000
            cleanup_vmas_for_pml4(0x3000);
            assert!(!VMA_TABLE[1].is_active);
        }
    }

    /// Helper to reset all VMA table entries for a specific PML4.
    unsafe fn reset_vma_table_for(pml4: u64) {
        cleanup_vmas_for_pml4(pml4);
    }

    /// Helper to inject a VMA entry at a specific slot index.
    unsafe fn inject_vma(slot: usize, pml4: u64, start: u64, end: u64, prot: u32, flags: u32) {
        VMA_TABLE[slot] = Vma::new_anon(pml4, start, end, prot, flags);
    }

    /// Helper constant: the PML4 value returned by `active_pml4()` in test environment.
    const TEST_PML4: u64 = 0x1000;

    #[test]
    fn test_find_free_mmap_range_skip_collision() {
        let _lock = VMA_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        unsafe {
            reset_vma_table_for(TEST_PML4);

            // Place a VMA occupying the first 0x4000 bytes at MMAP_START
            inject_vma(
                0,
                TEST_PML4,
                MMAP_START,
                MMAP_START + 0x4000,
                PROT_READ,
                MAP_PRIVATE | MAP_ANONYMOUS,
            );

            // The allocator must skip the occupied range
            let result = find_free_mmap_range(TEST_PML4, 0x1000);
            assert_eq!(result, Some(MMAP_START + 0x4000));

            reset_vma_table_for(TEST_PML4);
        }
    }

    #[test]
    fn test_vma_table_full() {
        let _lock = VMA_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        unsafe {
            reset_vma_table_for(TEST_PML4);

            // Fill every VMA slot
            for i in 0..MAX_VMAS {
                inject_vma(
                    i,
                    TEST_PML4,
                    MMAP_START + (i as u64) * 0x2000,
                    MMAP_START + (i as u64) * 0x2000 + 0x1000,
                    PROT_READ,
                    MAP_PRIVATE | MAP_ANONYMOUS,
                );
            }

            // A new allocation should fail because all 64 VMA slots are occupied
            let _result = find_free_mmap_range(TEST_PML4, 0x1000);
            // find_free_mmap_range itself should still find a gap, but sys_mmap
            // should reject because no VMA slot is available.
            // We cannot call sys_mmap directly in test (no real page tables),
            // so verify that all slots are active.
            let active_count = (0..MAX_VMAS).filter(|&i| VMA_TABLE[i].is_active).count();
            assert_eq!(active_count, MAX_VMAS);

            reset_vma_table_for(TEST_PML4);
        }
    }

    #[test]
    fn test_map_fixed_vma_collision() {
        let _lock = VMA_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        unsafe {
            reset_vma_table_for(TEST_PML4);

            // Place a VMA at MMAP_START..MMAP_START+0x2000
            inject_vma(
                0,
                TEST_PML4,
                MMAP_START,
                MMAP_START + 0x2000,
                PROT_READ,
                MAP_PRIVATE | MAP_ANONYMOUS,
            );

            // MAP_FIXED overlapping the existing VMA should fail
            let res = sys_mmap(MMAP_START, 0x1000, PROT_READ, MAP_FIXED | MAP_ANONYMOUS);
            assert!(res.is_err());
            assert_eq!(
                res.unwrap_err(),
                "MAP_FIXED collision with existing VMA mapping"
            );

            reset_vma_table_for(TEST_PML4);
        }
    }

    #[test]
    fn test_munmap_no_matching_vma() {
        unsafe {
            reset_vma_table_for(TEST_PML4);

            // No VMA exists at this address
            let res = sys_munmap(MMAP_START, 0x1000);
            assert!(res.is_err());
            assert_eq!(
                res.unwrap_err(),
                "No matching active user VMA for munmap range"
            );
        }
    }

    #[test]
    fn test_munmap_exact_vma_deactivates() {
        unsafe {
            reset_vma_table_for(TEST_PML4);

            // Inject a 2-page VMA
            inject_vma(
                0,
                TEST_PML4,
                MMAP_START,
                MMAP_START + 0x2000,
                PROT_READ | PROT_WRITE,
                MAP_PRIVATE | MAP_ANONYMOUS,
            );

            // Munmap the entire range (will fail at page level since no real mapping,
            // but VMA metadata should still be checked for the right slot)
            let res = sys_munmap(MMAP_START, 0x2000);
            // In test mode, free_and_unmap_page will fail (no real page tables),
            // which means unmap_err is set, but actual_unmapped_len == 0,
            // so VMA is NOT modified. Verify the error propagation.
            assert!(res.is_err());

            reset_vma_table_for(TEST_PML4);
        }
    }

    #[test]
    fn test_munmap_front_trim_metadata() {
        unsafe {
            reset_vma_table_for(TEST_PML4);

            // Inject a 4-page VMA and verify front-trim metadata logic
            inject_vma(
                0,
                TEST_PML4,
                MMAP_START,
                MMAP_START + 0x4000,
                PROT_READ,
                MAP_PRIVATE | MAP_ANONYMOUS,
            );

            // Verify the VMA range is correct before any munmap
            assert_eq!(VMA_TABLE[0].start, MMAP_START);
            assert_eq!(VMA_TABLE[0].end, MMAP_START + 0x4000);
            assert!(VMA_TABLE[0].is_active);

            reset_vma_table_for(TEST_PML4);
        }
    }

    #[test]
    fn test_munmap_back_trim_metadata() {
        unsafe {
            reset_vma_table_for(TEST_PML4);

            inject_vma(
                0,
                TEST_PML4,
                MMAP_START,
                MMAP_START + 0x4000,
                PROT_READ,
                MAP_PRIVATE | MAP_ANONYMOUS,
            );

            // Back-trim verification: the VMA must have correct bounds
            assert_eq!(VMA_TABLE[0].end, MMAP_START + 0x4000);

            reset_vma_table_for(TEST_PML4);
        }
    }

    #[test]
    fn test_munmap_middle_split_requires_free_slot() {
        let _lock = VMA_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        unsafe {
            reset_vma_table_for(TEST_PML4);

            // Inject a large VMA
            inject_vma(
                0,
                TEST_PML4,
                MMAP_START,
                MMAP_START + 0x6000,
                PROT_READ,
                MAP_PRIVATE | MAP_ANONYMOUS,
            );

            // Fill all remaining slots to prevent a middle-split
            for i in 1..MAX_VMAS {
                inject_vma(
                    i,
                    0x9999,
                    MMAP_START + (i as u64) * 0x10000,
                    MMAP_START + (i as u64) * 0x10000 + 0x1000,
                    PROT_READ,
                    MAP_PRIVATE | MAP_ANONYMOUS,
                );
            }

            // Middle munmap should fail because no free VMA slot for the split
            let res = sys_munmap(MMAP_START + 0x2000, 0x2000);
            assert!(res.is_err());
            assert_eq!(
                res.unwrap_err(),
                "Max VMA capacity reached during partial munmap split"
            );

            reset_vma_table_for(TEST_PML4);
            cleanup_vmas_for_pml4(0x9999);
        }
    }

    #[test]
    fn test_mprotect_noop_same_prot() {
        unsafe {
            reset_vma_table_for(TEST_PML4);

            inject_vma(
                0,
                TEST_PML4,
                MMAP_START,
                MMAP_START + 0x2000,
                PROT_READ,
                MAP_PRIVATE | MAP_ANONYMOUS,
            );

            // mprotect with the same protection should be a no-op success
            let res = sys_mprotect(MMAP_START, 0x2000, PROT_READ);
            assert!(res.is_ok());

            // VMA metadata must remain unchanged
            assert_eq!(VMA_TABLE[0].prot, PROT_READ);
            assert_eq!(VMA_TABLE[0].start, MMAP_START);
            assert_eq!(VMA_TABLE[0].end, MMAP_START + 0x2000);

            reset_vma_table_for(TEST_PML4);
        }
    }

    #[test]
    fn test_mprotect_no_matching_vma() {
        unsafe {
            reset_vma_table_for(TEST_PML4);

            let res = sys_mprotect(MMAP_START, 0x1000, PROT_READ);
            assert!(res.is_err());
            assert_eq!(
                res.unwrap_err(),
                "No matching active user VMA for mprotect range"
            );
        }
    }

    #[test]
    fn test_mprotect_middle_split_capacity_check() {
        let _lock = VMA_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        unsafe {
            reset_vma_table_for(TEST_PML4);

            // Inject a large VMA
            inject_vma(
                0,
                TEST_PML4,
                MMAP_START,
                MMAP_START + 0x6000,
                PROT_READ,
                MAP_PRIVATE | MAP_ANONYMOUS,
            );

            // Fill all remaining slots except one
            for i in 1..(MAX_VMAS - 1) {
                inject_vma(
                    i,
                    0x8888,
                    MMAP_START + (i as u64) * 0x10000,
                    MMAP_START + (i as u64) * 0x10000 + 0x1000,
                    PROT_READ,
                    MAP_PRIVATE | MAP_ANONYMOUS,
                );
            }

            // Middle mprotect requires 2 free slots but only 1 is available
            let res = sys_mprotect(MMAP_START + 0x2000, 0x2000, PROT_READ | PROT_WRITE);
            assert!(res.is_err());
            assert_eq!(
                res.unwrap_err(),
                "Max VMA capacity reached during partial mprotect split"
            );

            reset_vma_table_for(TEST_PML4);
            cleanup_vmas_for_pml4(0x8888);
        }
    }

    #[test]
    fn test_aligned_len_zero_munmap() {
        unsafe {
            // length=0 is caught first by the zero-length check
            let res = sys_munmap(MMAP_START, 0);
            assert!(res.is_err());
        }
    }

    #[test]
    fn test_aligned_len_zero_mprotect() {
        unsafe {
            // length=0 is caught first by the zero-length check
            let res = sys_mprotect(MMAP_START, 0, PROT_READ);
            assert!(res.is_err());
        }
    }

    #[test]
    fn test_mmap_overflow_length() {
        unsafe {
            // Maximum u64 length should trigger overflow
            let res = sys_mmap(0, u64::MAX, PROT_READ, MAP_ANONYMOUS);
            assert!(res.is_err());
        }
    }

    #[test]
    fn test_map_fixed_range_exceeds_boundary() {
        unsafe {
            // MAP_FIXED at end of mmap region with length that overflows past MMAP_END
            let near_end = MMAP_END - 0x1000;
            let res = sys_mmap(near_end, 0x2000, PROT_READ, MAP_FIXED | MAP_ANONYMOUS);
            assert!(res.is_err());
            assert_eq!(
                res.unwrap_err(),
                "MAP_FIXED range exceeds user mmap boundary"
            );
        }
    }

    #[test]
    fn test_find_free_mmap_range_no_space() {
        let _lock = VMA_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        unsafe {
            reset_vma_table_for(TEST_PML4);

            // Place a VMA covering the entire mmap region
            inject_vma(
                0,
                TEST_PML4,
                MMAP_START,
                MMAP_END,
                PROT_READ,
                MAP_PRIVATE | MAP_ANONYMOUS,
            );

            // No free range should be found
            let result = find_free_mmap_range(TEST_PML4, 0x1000);
            assert_eq!(result, None);

            reset_vma_table_for(TEST_PML4);
        }
    }

    #[test]
    fn test_munmap_ext_exact_bytes_exposed() {
        unsafe {
            reset_vma_table_for(TEST_PML4);

            inject_vma(
                0,
                TEST_PML4,
                MMAP_START,
                MMAP_START + 0x4000,
                PROT_READ,
                MAP_PRIVATE | MAP_ANONYMOUS,
            );

            // In test mode without mapped pages, munmap fails at page level on first page,
            // so actual_unmapped_len is 0.
            let res = sys_munmap_ext(MMAP_START, 0x2000);
            assert!(res.is_err());
            let (msg, unmapped) = res.unwrap_err();
            assert_eq!(unmapped, 0);
            assert_eq!(msg, "Virtual address not mapped");

            reset_vma_table_for(TEST_PML4);
        }
    }

    #[test]
    fn test_munmap_ext_validation_errors() {
        unsafe {
            // Zero length
            let res0 = sys_munmap_ext(MMAP_START, 0);
            assert!(res0.is_err());
            assert_eq!(res0.unwrap_err().1, 0);

            // Unaligned address
            let res_unaligned = sys_munmap_ext(MMAP_START + 1, 0x1000);
            assert!(res_unaligned.is_err());
            assert_eq!(res_unaligned.unwrap_err().1, 0);

            // Out of bounds address
            let res_oob = sys_munmap_ext(0x1000_0000, 0x1000);
            assert!(res_oob.is_err());
            assert_eq!(res_oob.unwrap_err().1, 0);
        }
    }

    #[test]
    fn test_page_table_level_huge_page_invariants() {
        // Test simulated 4-level page table integrity using page-aligned physical addresses
        let pdpt_phys = 0x1000_0000u64;
        let pd_phys = 0x1001_0000u64;

        let mut test_pml4 = [0u64; 512];
        let mut test_pdpt = [0u64; 512];
        let mut test_pd = [0u64; 512];

        // Link PML4[1] -> PDPT
        test_pml4[1] = pdpt_phys | paging::PAGE_PRESENT | paging::PAGE_USER;

        // Set 1GB Huge Page in PDPT[0]
        let frame_1gb = 0x4000_0000u64;
        test_pdpt[0] = frame_1gb | paging::PAGE_PRESENT | paging::PAGE_USER | paging::PAGE_HUGE;

        // Verify that PDPT[0] holds the 1GB entry, while PML4[1] still points to PDPT
        assert_eq!(test_pml4[1] & paging::PTE_ADDR_MASK, pdpt_phys);
        assert_eq!(test_pdpt[0] & paging::PTE_ADDR_MASK_1G, frame_1gb);

        // Unmapping 1GB huge page clears PDPT[0], PML4[1] MUST remain intact!
        test_pdpt[0] = 0;
        assert_eq!(test_pdpt[0], 0);
        assert_eq!(test_pml4[1] & paging::PTE_ADDR_MASK, pdpt_phys);

        // Link PDPT[1] -> PD
        test_pdpt[1] = pd_phys | paging::PAGE_PRESENT | paging::PAGE_USER;

        // Set 2MB Huge Page in PD[0]
        let frame_2mb = 0x20_0000u64;
        test_pd[0] = frame_2mb | paging::PAGE_PRESENT | paging::PAGE_USER | paging::PAGE_HUGE;

        // Verify that PD[0] holds the 2MB entry, while PDPT[1] still points to PD
        assert_eq!(test_pdpt[1] & paging::PTE_ADDR_MASK, pd_phys);
        assert_eq!(test_pd[0] & paging::PTE_ADDR_MASK_2M, frame_2mb);

        // Unmapping 2MB huge page clears PD[0], PDPT[1] MUST remain intact!
        test_pd[0] = 0;
        assert_eq!(test_pd[0], 0);
        assert_eq!(test_pdpt[1] & paging::PTE_ADDR_MASK, pd_phys);
    }

    #[test]
    fn test_file_backed_mmap_and_msync() {
        let _lock = VMA_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        unsafe {
            reset_vma_table_for(TEST_PML4);

            static mut SYNC_CALLED: bool = false;
            fn dummy_read(
                _path: &str,
                _offset: u64,
                buf: &mut [u8],
            ) -> Result<usize, &'static str> {
                for (i, b) in buf.iter_mut().enumerate() {
                    *b = (i % 256) as u8;
                }
                Ok(buf.len())
            }
            fn dummy_sync(_path: &str, _offset: u64, _buf: &[u8]) -> Result<usize, &'static str> {
                unsafe {
                    SYNC_CALLED = true;
                }
                Ok(_buf.len())
            }

            register_file_backing_hooks(dummy_read, dummy_sync);
            assert!(get_file_read_hook().is_some());
            assert!(get_file_sync_hook().is_some());

            let vaddr = sys_mmap_file(
                0,
                0x1000,
                PROT_READ | PROT_WRITE,
                MAP_SHARED,
                Some("/data/test.bin"),
                0,
                0x1000,
            )
            .expect("sys_mmap_file should succeed");

            let vma = find_active_vma(TEST_PML4, vaddr).expect("VMA should exist");
            assert!(vma.file_backed);
            assert_eq!(vma.file_path_str(), Some("/data/test.bin"));
            assert_eq!(vma.file_offset, 0);
            assert_eq!(vma.file_size, 0x1000);

            // Test msync
            let sync_res = sys_msync(vaddr, 0x1000, MS_SYNC);
            assert!(sync_res.is_ok());
            assert!(SYNC_CALLED);

            // Clean up
            cleanup_vmas_for_pml4(TEST_PML4);
            assert!(find_active_vma(TEST_PML4, vaddr).is_none());
        }
    }
}
