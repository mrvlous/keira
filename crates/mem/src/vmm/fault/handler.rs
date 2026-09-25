// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Page Fault (#PF, Interrupt 14) handling, stack auto-growth, and demand paging.

use super::super::area::{find_active_vma, get_file_read_hook, PROT_EXEC, PROT_READ, PROT_WRITE};
use super::super::mapping::map_page;
use super::super::table::{
    active_pml4, get_pte_mut_in_pml4, PAGE_COW, PAGE_NO_EXECUTE, PAGE_PRESENT, PAGE_USER,
    PAGE_WRITABLE, PTE_ADDR_MASK,
};
use crate::pmm;
use keira_arch::cpu::invlpg;

#[cfg(target_arch = "x86")]
pub const USER_STACK_TOP: u64 = 0x07FFF000;
#[cfg(target_arch = "x86")]
pub const USER_STACK_BOTTOM: u64 = 0x07F80000;

#[cfg(target_arch = "x86_64")]
pub const USER_STACK_TOP: u64 = 0x7FFFFFE00000;
#[cfg(target_arch = "x86_64")]
pub const USER_STACK_BOTTOM: u64 = 0x7FFFFFD80000;

/// Processes an active page fault interrupt vector.
///
/// Resolves recoverable page faults via:
/// 1. Copy-on-Write (COW) page frame replication.
/// 2. User stack on-demand growth.
/// 3. VMA demand paging for anonymous and file-backed regions.
///
/// Returns `true` if the page fault was handled and execution can resume,
/// or `false` if the fault represents an unrecoverable access violation.
///
/// # Safety
///
/// Reads and modifies live processor page tables and invalidates the processor TLB.
pub unsafe fn handle_page_fault(cr2: u64, error_code: u64, rsp: u64) -> bool {
    let pml4 = active_pml4();
    let is_present = (error_code & 1) != 0;
    let is_write = (error_code & 2) != 0;
    let is_instruction = (error_code & 16) != 0;

    let fault_page = cr2 & !(pmm::PAGE_SIZE - 1);
    if fault_page == 0 {
        return false;
    }

    if is_present && is_write {
        if let Some(pte_ptr) = get_pte_mut_in_pml4(pml4, fault_page) {
            let pte = *pte_ptr;
            if (pte & PAGE_COW) != 0 {
                if let Some(new_frame) = pmm::alloc_frame() {
                    let src_ptr = fault_page as *const u8;
                    let dst_ptr = new_frame as *mut u8;
                    core::ptr::copy_nonoverlapping(src_ptr, dst_ptr, pmm::PAGE_SIZE as usize);

                    *pte_ptr = (new_frame & PTE_ADDR_MASK)
                        | (pte & !(PTE_ADDR_MASK | PAGE_COW))
                        | PAGE_WRITABLE;

                    invlpg(fault_page as usize);
                    return true;
                }
            }
        }
        return false;
    }

    if is_present {
        return false;
    }

    let is_stack_fault = cr2 >= USER_STACK_BOTTOM
        && cr2 < USER_STACK_TOP
        && (cr2 >= rsp.saturating_sub(256) || rsp >= USER_STACK_BOTTOM);

    if is_stack_fault {
        if let Some(frame) = pmm::alloc_frame() {
            core::ptr::write_bytes(frame as *mut u8, 0, pmm::PAGE_SIZE as usize);

            #[cfg(target_arch = "x86_64")]
            let flags = PAGE_PRESENT | PAGE_WRITABLE | PAGE_USER | PAGE_NO_EXECUTE;
            #[cfg(not(target_arch = "x86_64"))]
            let flags = PAGE_PRESENT | PAGE_WRITABLE | PAGE_USER;
            if map_page(fault_page, frame, flags).is_ok() {
                invlpg(fault_page as usize);
                return true;
            } else {
                pmm::free_frame(frame);
            }
        }
        return false;
    }

    if let Some(vma) = find_active_vma(pml4, cr2) {
        if is_write && (vma.prot & PROT_WRITE) == 0 {
            return false;
        }
        if is_instruction && (vma.prot & PROT_EXEC) == 0 {
            return false;
        }
        if !is_write && !is_instruction && (vma.prot & (PROT_READ | PROT_EXEC)) == 0 {
            return false;
        }

        if let Some(frame) = pmm::alloc_frame() {
            core::ptr::write_bytes(frame as *mut u8, 0, pmm::PAGE_SIZE as usize);

            if vma.file_backed {
                if let (Some(read_fn), Some(path)) = (get_file_read_hook(), vma.file_path_str()) {
                    let page_delta = fault_page - vma.start;
                    let file_offset = vma.file_offset + page_delta;
                    if file_offset < vma.file_size {
                        let to_read =
                            core::cmp::min(pmm::PAGE_SIZE, vma.file_size - file_offset) as usize;
                        let dst_slice = core::slice::from_raw_parts_mut(frame as *mut u8, to_read);
                        let _ = read_fn(path, file_offset, dst_slice);
                    }
                }
            }

            let mut flags = PAGE_PRESENT | PAGE_USER;
            if (vma.prot & PROT_WRITE) != 0 {
                flags |= PAGE_WRITABLE;
            }
            if (vma.prot & PROT_EXEC) == 0 {
                flags |= PAGE_NO_EXECUTE;
            }

            if map_page(fault_page, frame, flags).is_ok() {
                invlpg(fault_page as usize);
                return true;
            } else {
                pmm::free_frame(frame);
            }
        }
    }

    false
}
