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

//! User space virtual address boundaries, range checks and pointer validation routines.

use crate::user_copy::errno::EFAULT;

#[cfg(all(target_arch = "x86_64", not(test)))]
use keira_mem::{pmm, vmm};

pub const USER_MIN_ADDR: u64 = 0x10000;

#[cfg(target_arch = "x86_64")]
pub const USER_MAX_ADDR: u64 = 0x0000_7FFF_FFFF_FFFF;

#[cfg(target_arch = "x86")]
pub const USER_MAX_ADDR: u64 = 0xBFFF_FFFF;

/// Validates that `[ptr, ptr + len)` is strictly contained within user space virtual address limits.
#[inline]
pub fn check_user_bounds(ptr: u64, len: u64) -> Result<u64, i64> {
    if len == 0 {
        return Ok(ptr);
    }

    let end = match ptr.checked_add(len) {
        Some(e) => e,
        None => return Err(EFAULT),
    };

    if ptr < USER_MIN_ADDR || end > USER_MAX_ADDR {
        return Err(EFAULT);
    }

    Ok(end)
}

/// Attempts to fault-in an unmapped user page via VMA descriptors or heap expansion.
///
/// # Safety
/// Directly manipulates active page tables, allocates physical frames and invalidates TLB.
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

/// Fallback dummy fault handler for 32-bit and test targets.
///
/// # Safety
/// Always returns false without modifying memory.
#[cfg(any(target_arch = "x86", test))]
pub unsafe fn try_fault_user_page(_vaddr: u64, _require_writable: bool) -> bool {
    false
}

/// Validates that a user memory buffer is completely contained within valid user virtual address space.
///
/// # Safety
/// Traverses and checks user virtual page mappings directly.
pub unsafe fn validate_user_ptr(ptr: u64, len: u64, require_writable: bool) -> Result<(), i64> {
    if len == 0 {
        return Ok(());
    }

    let end = check_user_bounds(ptr, len)?;

    #[cfg(all(target_arch = "x86_64", not(test)))]
    {
        let mut page_start = ptr & !(pmm::PAGE_SIZE - 1);
        let page_end = match end.checked_add(pmm::PAGE_SIZE - 1) {
            Some(e) => e & !(pmm::PAGE_SIZE - 1),
            None => return Err(EFAULT),
        };

        while page_start < page_end {
            if !vmm::is_user_page_mapped(page_start, require_writable)
                && !try_fault_user_page(page_start, require_writable)
            {
                return Err(EFAULT);
            }
            page_start += pmm::PAGE_SIZE;
        }
    }

    #[cfg(any(target_arch = "x86", test))]
    {
        let _ = (end, require_writable);
    }

    Ok(())
}

