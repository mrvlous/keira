// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Centralized safe user space memory validation, range checking, and copying primitives.

#[cfg(target_arch = "x86_64")]
use keira_mem::{pmm, vmm};

pub const EPERM: i64 = 1;
pub const ENOENT: i64 = 2;
pub const ESRCH: i64 = 3;
pub const EINTR: i64 = 4;
pub const EIO: i64 = 5;
pub const EBADF: i64 = 9;
pub const ECHILD: i64 = 10;
pub const EAGAIN: i64 = 11;
pub const ENOMEM: i64 = 12;
pub const EACCES: i64 = 13;
pub const EFAULT: i64 = 14;
pub const EEXIST: i64 = 17;
pub const EINVAL: i64 = 22;
pub const EMFILE: i64 = 24;
pub const ENOSYS: i64 = 38;

pub const USER_MIN_ADDR: u64 = 0x10000;
#[cfg(target_arch = "x86_64")]
pub const USER_MAX_ADDR: u64 = 0x0000_7FFF_FFFF_FFFF;
#[cfg(target_arch = "x86")]
pub const USER_MAX_ADDR: u64 = 0xBFFF_FFFF; // 3GB user virtual address space

/// Convert POSIX errno to unsigned syscall return value (-errno as u64).
#[inline]
pub fn errno_to_ret(err: i64) -> u64 {
    (-err) as u64
}

/// Validate that a user pointer range resides strictly in user virtual memory and is mapped.
pub unsafe fn validate_user_ptr(ptr: u64, len: u64, require_writable: bool) -> Result<(), i64> {
    if len == 0 {
        return Ok(());
    }

    let end = match ptr.checked_add(len) {
        Some(e) => e,
        None => return Err(EFAULT),
    };

    if ptr < USER_MIN_ADDR || end > USER_MAX_ADDR {
        return Err(EFAULT);
    }

    #[cfg(target_arch = "x86_64")]
    {
        // Verify all pages spanned by the buffer are mapped with proper permissions (overflow-checked)
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

    #[cfg(target_arch = "x86")]
    {
        let _ = require_writable;
    }

    Ok(())
}

/// Safely copy data from kernel buffer to user space virtual address.
pub unsafe fn copy_to_user(dest_user_ptr: u64, src: &[u8]) -> Result<(), i64> {
    if src.is_empty() {
        return Ok(());
    }

    validate_user_ptr(dest_user_ptr, src.len() as u64, true)?;
    core::ptr::copy_nonoverlapping(src.as_ptr(), dest_user_ptr as *mut u8, src.len());
    Ok(())
}

/// Safely copy data from user space virtual address into kernel buffer.
pub unsafe fn copy_from_user(dest: &mut [u8], src_user_ptr: u64) -> Result<(), i64> {
    if dest.is_empty() {
        return Ok(());
    }

    validate_user_ptr(src_user_ptr, dest.len() as u64, false)?;
    core::ptr::copy_nonoverlapping(src_user_ptr as *const u8, dest.as_mut_ptr(), dest.len());
    Ok(())
}

/// Safely read a null-terminated string from user space memory, checking each page mapping before read.
pub unsafe fn read_user_string(ptr: *const u8, buf: &mut [u8]) -> Result<usize, i64> {
    if ptr.is_null() {
        return Err(EFAULT);
    }

    let mut len = 0;
    let max_len = buf.len();
    #[cfg(target_arch = "x86_64")]
    let mut current_page: u64 = 0;

    while len < max_len - 1 {
        let addr = match (ptr as u64).checked_add(len as u64) {
            Some(a) => a,
            None => return Err(EFAULT),
        };

        if addr < USER_MIN_ADDR || addr > USER_MAX_ADDR {
            return Err(EFAULT);
        }

        #[cfg(target_arch = "x86_64")]
        {
            let page_addr = addr & !(pmm::PAGE_SIZE - 1);
            if page_addr != current_page {
                if !vmm::is_user_page_mapped(page_addr, false)
                    && !try_fault_user_page(page_addr, false)
                {
                    return Err(EFAULT);
                }
                current_page = page_addr;
            }
        }

        let c = *ptr.add(len);
        if c == 0 {
            break;
        }
        buf[len] = c;
        len += 1;
    }

    Ok(len)
}

#[cfg(target_arch = "x86_64")]
unsafe fn try_fault_user_page(vaddr: u64, require_writable: bool) -> bool {
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
