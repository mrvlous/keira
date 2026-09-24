// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Demand paging and stack auto-growth resolution during user space pointer validation.

#[cfg(all(target_arch = "x86_64", not(test)))]
use keira_mem::{pmm, vmm};

/// Attempts to fault-in an unmapped user page via VMA descriptors or heap expansion.
#[cfg(all(target_arch = "x86_64", not(test)))]
pub unsafe fn try_fault_user_page(vaddr: u64, require_writable: bool) -> bool {
    let pml4 = vmm::active_pml4();
    if let Some(vma) = vmm::find_active_vma(pml4, vaddr) {
        if require_writable && (vma.prot & vmm::PROT_WRITE) == 0 {
            return false;
        }
        let error_code = if require_writable { 2 } else { 0 };
        return vmm::handle_page_fault(vaddr, error_code, 0);
    }

    let task_idx = keira_task::scheduler::CURRENT_TASK_IDX;
    if let Some(ref t) = keira_task::scheduler::TASKS[task_idx] {
        if vaddr >= t.program_break_start && vaddr < t.program_break {
            let fault_page = vaddr & !(pmm::PAGE_SIZE - 1);
            if let Some(frame) = pmm::alloc_frame() {
                core::ptr::write_bytes(frame as *mut u8, 0, pmm::PAGE_SIZE as usize);
                let flags = vmm::PAGE_PRESENT | vmm::PAGE_WRITABLE | vmm::PAGE_USER;
                if vmm::map_page(fault_page, frame, flags).is_ok() {
                    keira_arch::cpu::invlpg(fault_page as usize);
                    return true;
                } else {
                    pmm::free_frame(frame);
                }
            }
        }
    }

    let error_code = if require_writable { 2 } else { 0 };
    vmm::handle_page_fault(vaddr, error_code, vaddr)
}

#[cfg(any(target_arch = "x86", test))]
pub unsafe fn try_fault_user_page(_vaddr: u64, _require_writable: bool) -> bool {
    false
}
