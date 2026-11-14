// SPDX-License-Identifier: GPL-2.0-only
/*
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra <mrvlous@proton.me>
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 */

//! Virtual Memory Area (VMA) allocation, memory map descriptors and operations.

pub mod descriptor {
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

    #[cfg(target_arch = "x86")]
    pub const MMAP_START: u64 = 0x0400_0000;
    #[cfg(target_arch = "x86")]
    pub const MMAP_END: u64 = 0x0700_0000;

    #[cfg(not(target_arch = "x86"))]
    pub const MMAP_START: u64 = 0x5000_0000_0000;
    #[cfg(not(target_arch = "x86"))]
    pub const MMAP_END: u64 = 0x7000_0000_0000;

    pub const MAX_VMAS: usize = 64;

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
        /// Constructs a vacant, inactive VMA descriptor slot.
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

        /// Constructs an active anonymous VMA spanning `[start, end)`.
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

        /// Retrieves the UTF-8 file path string if this VMA is file-backed.
        pub fn file_path_str(&self) -> Option<&str> {
            if !self.file_backed || self.file_path_len == 0 || self.file_path_len > 128 {
                None
            } else {
                core::str::from_utf8(&self.file_path[..self.file_path_len]).ok()
            }
        }
    }
}

pub mod hooks {
    pub type FileReadHook =
        fn(path: &str, offset: u64, buf: &mut [u8]) -> Result<usize, &'static str>;

    pub type FileSyncHook = fn(path: &str, offset: u64, buf: &[u8]) -> Result<usize, &'static str>;

    pub(crate) static mut FILE_READ_HOOK: Option<FileReadHook> = None;
    pub(crate) static mut FILE_SYNC_HOOK: Option<FileSyncHook> = None;

    pub fn register_file_backing_hooks(read_hook: FileReadHook, sync_hook: FileSyncHook) {
        unsafe {
            FILE_READ_HOOK = Some(read_hook);
            FILE_SYNC_HOOK = Some(sync_hook);
        }
    }

    pub fn get_file_read_hook() -> Option<FileReadHook> {
        unsafe { FILE_READ_HOOK }
    }

    pub fn get_file_sync_hook() -> Option<FileSyncHook> {
        unsafe { FILE_SYNC_HOOK }
    }
}

pub mod invariants {
    use super::super::table::{
        get_pte_in_pml4, PAGE_NO_EXECUTE, PAGE_PRESENT, PAGE_USER, PAGE_WRITABLE,
    };
    #[cfg(not(test))]
    use super::super::table::{PAGE_HUGE, PTE_ADDR_MASK};
    #[cfg(not(test))]
    use super::descriptor::{MMAP_END, MMAP_START};
    use super::descriptor::{PROT_EXEC, PROT_WRITE};
    use super::table::VMA_TABLE;
    use crate::pmm;

    pub unsafe fn verify_vma_pte_invariants(pml4_phys: u64) -> Result<(), &'static str> {
        if pml4_phys == 0 {
            return Ok(());
        }

        for vma in VMA_TABLE.iter() {
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
                                return Err(
                                    "VMA invariant violation: Read-only VMA has writable PTE",
                                );
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

        #[cfg(not(test))]
        {
            let pml4 = pml4_phys as *const u64;

            for pml4_idx in 1..256 {
                let pml4_entry = *pml4.add(pml4_idx);
                if (pml4_entry & PAGE_PRESENT) == 0 {
                    continue;
                }
                let pdpt_phys = pml4_entry & PTE_ADDR_MASK;
                let pdpt = pdpt_phys as *const u64;

                for pdpt_idx in 0..512 {
                    let pdpt_entry = *pdpt.add(pdpt_idx);
                    if (pdpt_entry & PAGE_PRESENT) == 0 {
                        continue;
                    }
                    if (pdpt_entry & PAGE_HUGE) != 0 {
                        let page_start = ((pml4_idx as u64) << 39) | ((pdpt_idx as u64) << 30);
                        let page_end = page_start + 0x4000_0000;
                        if page_start < MMAP_END && page_end > MMAP_START {
                            let mut in_vma = false;
                            for v in VMA_TABLE.iter() {
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

                    let pd_phys = pdpt_entry & PTE_ADDR_MASK;
                    let pd = pd_phys as *const u64;

                    for pd_idx in 0..512 {
                        let pd_entry = *pd.add(pd_idx);
                        if (pd_entry & PAGE_PRESENT) == 0 {
                            continue;
                        }
                        if (pd_entry & PAGE_HUGE) != 0 {
                            let page_start = ((pml4_idx as u64) << 39)
                                | ((pdpt_idx as u64) << 30)
                                | ((pd_idx as u64) << 21);
                            let page_end = page_start + 0x20_0000;
                            if page_start < MMAP_END && page_end > MMAP_START {
                                let mut in_vma = false;
                                for v in VMA_TABLE.iter() {
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

                        let pt_phys = pd_entry & PTE_ADDR_MASK;
                        let pt = pt_phys as *const u64;

                        for pt_idx in 0..512 {
                            let pt_entry = *pt.add(pt_idx);
                            if (pt_entry & PAGE_PRESENT) != 0 && (pt_entry & PAGE_USER) != 0 {
                                let page_start = ((pml4_idx as u64) << 39)
                                    | ((pdpt_idx as u64) << 30)
                                    | ((pd_idx as u64) << 21)
                                    | ((pt_idx as u64) << 12);
                                let page_end = page_start + pmm::PAGE_SIZE;
                                if page_start < MMAP_END && page_end > MMAP_START {
                                    let mut in_vma = false;
                                    for v in VMA_TABLE.iter() {
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
}

pub mod table {
    use super::super::table::{USER_MAX_VADDR, USER_MIN_VADDR};
    use super::descriptor::{Vma, MAX_VMAS, MMAP_END, MMAP_START};

    pub static mut VMA_TABLE: [Vma; MAX_VMAS] = [Vma::empty(); MAX_VMAS];

    pub unsafe fn cleanup_vmas_for_pml4(pml4_phys: u64) {
        if pml4_phys == 0 && !cfg!(target_arch = "x86") {
            return;
        }
        for vma in VMA_TABLE.iter_mut() {
            if vma.is_active && vma.pml4_phys == pml4_phys {
                vma.is_active = false;
            }
        }
    }

    pub unsafe fn find_active_vma(pml4_phys: u64, addr: u64) -> Option<Vma> {
        if pml4_phys == 0 && !cfg!(target_arch = "x86") {
            return None;
        }
        for vma in VMA_TABLE.iter() {
            if vma.is_active && vma.pml4_phys == pml4_phys && (vma.start..vma.end).contains(&addr) {
                return Some(*vma);
            }
        }
        None
    }

    static mut MMAP_ASLR_OFFSET_HOOK: Option<fn() -> u64> = None;

    pub fn register_mmap_aslr_hook(hook: fn() -> u64) {
        unsafe {
            MMAP_ASLR_OFFSET_HOOK = Some(hook);
        }
    }

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
                if vma.is_active
                    && vma.pml4_phys == cur_pml4
                    && candidate < vma.end
                    && end > vma.start
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

    pub unsafe fn get_vmas_for_pml4(pml4_phys: u64, out_vmas: &mut [Vma]) -> usize {
        if pml4_phys == 0 && !cfg!(target_arch = "x86") {
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

    pub unsafe fn clone_vmas_for_pml4(
        parent_pml4: u64,
        child_pml4: u64,
    ) -> Result<(), &'static str> {
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

    pub unsafe fn total_active_vmas() -> usize {
        let mut count = 0;
        for vma in VMA_TABLE.iter() {
            if vma.is_active {
                count += 1;
            }
        }
        count
    }

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
}

pub mod ops {
    use super::super::mapping::{free_and_unmap_page, map_page, mprotect_page};
    use super::super::table::{
        active_pml4, is_page_mapped_in_pml4, PAGE_NO_EXECUTE, PAGE_PRESENT, PAGE_USER,
        PAGE_WRITABLE,
    };
    #[cfg(not(test))]
    use super::super::table::{get_pte_mut_in_pml4, PAGE_DIRTY};
    use super::descriptor::*;
    use super::hooks::{FILE_READ_HOOK, FILE_SYNC_HOOK};
    use super::table::{
        find_active_vma, find_free_mmap_range, validate_virt_addr_range, VMA_TABLE,
    };
    use crate::pmm;

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
            if !(MMAP_START..MMAP_END).contains(&hint_addr) {
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

            // 1. Check for collision with existing active VMAs in the same address space
            for vma in VMA_TABLE.iter() {
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
        for (i, vma) in VMA_TABLE.iter().enumerate() {
            if !vma.is_active {
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
                        let to_read =
                            core::cmp::min(pmm::PAGE_SIZE, file_size - cur_offset) as usize;
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

    pub unsafe fn sys_mmap(
        hint_addr: u64,
        length: u64,
        prot: u32,
        flags: u32,
    ) -> Result<u64, &'static str> {
        sys_mmap_file(hint_addr, length, prot, flags, None, 0, 0)
    }

    pub unsafe fn sys_munmap_ext(addr: u64, length: u64) -> Result<u64, (&'static str, u64)> {
        if length == 0 || !addr.is_multiple_of(pmm::PAGE_SIZE) {
            return Err(("Invalid address alignment or zero length for munmap", 0));
        }

        if !(MMAP_START..MMAP_END).contains(&addr) {
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
        for (i, vma) in VMA_TABLE.iter().enumerate() {
            if vma.is_active
                && vma.pml4_phys == cur_pml4
                && addr >= vma.start
                && target_end <= vma.end
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
            for (i, vma) in VMA_TABLE.iter().enumerate() {
                if !vma.is_active {
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

    pub unsafe fn sys_munmap(addr: u64, length: u64) -> Result<(), &'static str> {
        match sys_munmap_ext(addr, length) {
            Ok(_) => Ok(()),
            Err((err, _)) => Err(err),
        }
    }

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

        if !(MMAP_START..MMAP_END).contains(&addr) {
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
        for (i, vma) in VMA_TABLE.iter().enumerate() {
            if vma.is_active
                && vma.pml4_phys == cur_pml4
                && addr >= vma.start
                && target_end <= vma.end
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
            for (i, vma) in VMA_TABLE.iter().enumerate() {
                if !vma.is_active {
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

    pub unsafe fn sys_msync(addr: u64, length: u64, _flags: u32) -> Result<(), &'static str> {
        if length == 0 {
            return Err("Invalid msync length (0)");
        }
        if !addr.is_multiple_of(pmm::PAGE_SIZE) {
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
                                let write_len = core::cmp::min(
                                    pmm::PAGE_SIZE,
                                    vma.file_size - current_file_offset,
                                ) as usize;

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
                        core::cmp::min(pmm::PAGE_SIZE, vma.file_size - current_file_offset)
                            as usize;
                    let dummy = [0u8; 4096];
                    sync_fn(path, current_file_offset, &dummy[..write_len])?;
                }
            }
            curr_vaddr += pmm::PAGE_SIZE;
        }

        Ok(())
    }

    pub unsafe fn mmap_anonymous(
        hint_addr: u64,
        length: u64,
        prot: u32,
        flags: u32,
    ) -> Result<u64, &'static str> {
        sys_mmap(hint_addr, length, prot, flags | MAP_ANONYMOUS)
    }

    pub unsafe fn munmap_pages(addr: u64, length: u64) -> Result<(), &'static str> {
        sys_munmap(addr, length)
    }

    pub unsafe fn mprotect_pages(addr: u64, length: u64, prot: u32) -> Result<(), &'static str> {
        sys_mprotect(addr, length, prot)
    }

    pub unsafe fn madvise_pages(addr: u64, length: u64, _advice: u32) -> Result<(), &'static str> {
        validate_virt_addr_range(addr, length)
    }
}

pub use self::descriptor::*;
pub use self::hooks::*;
pub use self::invariants::*;
pub use self::ops::*;
pub use self::table::*;
