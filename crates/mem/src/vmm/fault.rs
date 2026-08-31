// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Robust Page Fault (#PF, Interrupt 14) handling, user stack auto-growth, and demand paging.

use super::mmap::{find_active_vma, PROT_EXEC, PROT_READ, PROT_WRITE};
use super::paging::{
    active_pml4, map_page, PAGE_NO_EXECUTE, PAGE_PRESENT, PAGE_USER, PAGE_WRITABLE,
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

/// Process a Page Fault interrupt. Returns `true` if the fault was resolved (e.g. via demand paging
/// or stack auto-growth) and execution should resume, or `false` if it is an unrecoverable fault.
pub unsafe fn handle_page_fault(cr2: u64, error_code: u64, rsp: u64) -> bool {
    let pml4 = active_pml4();
    let is_present = (error_code & 1) != 0;
    let is_write = (error_code & 2) != 0;
    let is_instruction = (error_code & 16) != 0;

    // We only resolve non-present pages for user mode (Demand Paging / Stack Growth)
    if is_present {
        return false;
    }

    // Align faulting address to page boundary
    let fault_page = cr2 & !(pmm::PAGE_SIZE - 1);

    // Guard against NULL pointer dereference or zero page access
    if fault_page == 0 {
        return false;
    }

    // 1. Check if faulting address is within the user stack growth window
    let is_stack_fault = cr2 >= USER_STACK_BOTTOM
        && cr2 < USER_STACK_TOP
        && (cr2 >= rsp.saturating_sub(256) || rsp >= USER_STACK_BOTTOM);

    if is_stack_fault {
        if let Some(frame) = pmm::alloc_frame() {
            // Zero the newly allocated stack page to avoid leaking previous memory
            core::ptr::write_bytes(frame as *mut u8, 0, pmm::PAGE_SIZE as usize);

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

    // 2. Check if faulting address resides inside an authorized user VMA (from sys_mmap)
    if let Some(vma) = find_active_vma(pml4, cr2) {
        // Validate access type against VMA protection flags
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
