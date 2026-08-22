// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Centralized safe user space memory validation, range checking, and copying primitives.

use keira_mem::pmm;
use keira_mem::vmm;

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
pub const ENOSYS: i64 = 38;

pub const USER_MIN_ADDR: u64 = 0x10000;
pub const USER_MAX_ADDR: u64 = 0x0000_7FFF_FFFF_FFFF;

/// Convert POSIX errno to 64-bit unsigned syscall return value (-errno as u64).
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

    // Verify all pages spanned by the buffer are mapped with proper permissions
    let mut page_start = ptr & !(pmm::PAGE_SIZE - 1);
    let page_end = (end + pmm::PAGE_SIZE - 1) & !(pmm::PAGE_SIZE - 1);

    while page_start < page_end {
        if !vmm::is_user_page_mapped(page_start, require_writable) {
            return Err(EFAULT);
        }
        page_start += pmm::PAGE_SIZE;
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

/// Safely read a null-terminated string from user space memory into kernel buffer.
pub unsafe fn read_user_string(ptr: *const u8, buf: &mut [u8]) -> Result<usize, i64> {
    if ptr.is_null() {
        return Err(EFAULT);
    }

    let mut len = 0;
    let max_len = buf.len();

    while len < max_len - 1 {
        let addr = (ptr as u64).checked_add(len as u64).ok_or(EFAULT)?;
        if addr < USER_MIN_ADDR || addr > USER_MAX_ADDR {
            return Err(EFAULT);
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
