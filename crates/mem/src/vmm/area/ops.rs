// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Memory mapping system call handlers and higher-level userland API interfaces.

use super::super::mapping::{free_and_unmap_page, map_page, mprotect_page};
use super::super::table::{
    active_pml4, is_page_mapped_in_pml4, PAGE_NO_EXECUTE, PAGE_PRESENT, PAGE_USER, PAGE_WRITABLE,
};
#[cfg(not(test))]
use super::super::table::{get_pte_mut_in_pml4, PAGE_DIRTY};
use super::descriptor::*;
use super::hooks::{FILE_READ_HOOK, FILE_SYNC_HOOK};
use super::table::{find_active_vma, find_free_mmap_range, validate_virt_addr_range, VMA_TABLE};
use crate::pmm;

/// Allocates and maps an anonymous or file-backed virtual memory region for user space with per-process VMA bookkeeping.
///
/// # Safety
///
/// Directly alters processor page table entries, allocates physical memory frames,
/// and mutates the global user virtual memory descriptor table.
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

/// Allocates and maps an anonymous virtual memory region for user space with per-process VMA bookkeeping.
///
/// # Safety
///
/// Modifies active page table mappings and manipulates the virtual memory area descriptor table.
pub unsafe fn sys_mmap(
    hint_addr: u64,
    length: u64,
    prot: u32,
    flags: u32,
) -> Result<u64, &'static str> {
    sys_mmap_file(hint_addr, length, prot, flags, None, 0, 0)
}

/// Extended virtual memory unmap operation returning the exact number of bytes successfully unmapped.
///
/// Returns `Ok(unmapped_bytes)` on complete success, or `Err((error_message, unmapped_bytes))` on partial failure.
///
/// # Safety
///
/// Unmaps pages from hardware page tables, flushes translation lookaside buffers (TLBs),
/// and reclaims underlying physical memory frames into the allocator free list.
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

/// Unmaps a user virtual memory region and releases underlying physical frames with VMA splitting.
///
/// # Safety
///
/// Directly alters hardware page table hierarchies and returns physical memory frames to the global allocator.
pub unsafe fn sys_munmap(addr: u64, length: u64) -> Result<(), &'static str> {
    match sys_munmap_ext(addr, length) {
        Ok(_) => Ok(()),
        Err((err, _)) => Err(err),
    }
}

/// Changes protection permissions on an existing mapped user memory range with VMA splitting.
///
/// Enforces W^X (Write XOR Execute) memory safety policy.
///
/// # Safety
///
/// Traverses and mutates active leaf page table entry permission bits (User, Writable, No-Execute).
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

/// Synchronizes a mapped memory region with its underlying file storage.
///
/// # Safety
///
/// Directly traverses page table entries and reads physical frame contents into the storage layer.
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
            if let Some(pte_ptr) = get_pte_mut_in_pml4(cur_pml4, curr_vaddr) {
                let pte = *pte_ptr;
                if (pte & PAGE_PRESENT) != 0 {
                    let is_dirty = (pte & PAGE_DIRTY) != 0 || (vma.prot & PROT_WRITE) != 0;
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

                        *pte_ptr = pte & !PAGE_DIRTY;
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

/// Allocates an anonymous user virtual memory region.
///
/// # Safety
///
/// Modifies active page tables and virtual memory area descriptors.
pub unsafe fn mmap_anonymous(
    hint_addr: u64,
    length: u64,
    prot: u32,
    flags: u32,
) -> Result<u64, &'static str> {
    sys_mmap(hint_addr, length, prot, flags | MAP_ANONYMOUS)
}

/// Unmaps a specified virtual memory page span.
///
/// # Safety
///
/// Alters hardware page tables and frees underlying physical frames.
pub unsafe fn munmap_pages(addr: u64, length: u64) -> Result<(), &'static str> {
    sys_munmap(addr, length)
}

/// Modifies memory protection bits for a virtual memory page range.
///
/// # Safety
///
/// Directly alters page table permissions and invalidates cached TLB translations.
pub unsafe fn mprotect_pages(addr: u64, length: u64, prot: u32) -> Result<(), &'static str> {
    sys_mprotect(addr, length, prot)
}

/// Advises the kernel regarding memory access patterns across a virtual address range.
///
/// # Safety
///
/// Reads and validates raw virtual address pointers.
pub unsafe fn madvise_pages(addr: u64, length: u64, _advice: u32) -> Result<(), &'static str> {
    validate_virt_addr_range(addr, length)
}
