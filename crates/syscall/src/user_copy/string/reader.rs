// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Safe string reading from user memory with page fault checks.

use crate::user_copy::errno::EFAULT;
use crate::user_copy::validate::{USER_MAX_ADDR, USER_MIN_ADDR};

#[cfg(all(target_arch = "x86_64", not(test)))]
use keira_mem::{pmm, vmm};

#[cfg(all(target_arch = "x86_64", not(test)))]
use crate::user_copy::validate::fault::try_fault_user_page;

/// Safely reads a null-terminated string from user space memory into a kernel buffer.
pub unsafe fn read_user_string(ptr: *const u8, buf: &mut [u8]) -> Result<usize, i64> {
    read_user_string_bounded(ptr, buf, buf.len())
}

/// Safely reads a null-terminated string up to `max_bytes` from user space memory into a kernel buffer.
pub unsafe fn read_user_string_bounded(
    ptr: *const u8,
    buf: &mut [u8],
    max_bytes: usize,
) -> Result<usize, i64> {
    if ptr.is_null() {
        return Err(EFAULT);
    }

    let mut len = 0;
    let limit = buf.len().min(max_bytes);
    if limit == 0 {
        return Ok(0);
    }

    #[cfg(all(target_arch = "x86_64", not(test)))]
    let mut current_page: u64 = 0;

    while len < limit.saturating_sub(1) {
        let addr = match (ptr as u64).checked_add(len as u64) {
            Some(a) => a,
            None => return Err(EFAULT),
        };

        if addr < USER_MIN_ADDR || addr > USER_MAX_ADDR {
            return Err(EFAULT);
        }

        #[cfg(all(target_arch = "x86_64", not(test)))]
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

    buf[len] = 0;
    Ok(len)
}
