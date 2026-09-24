// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! User space pointer validation checker functions.

#[cfg(all(target_arch = "x86_64", not(test)))]
use keira_mem::{pmm, vmm};

#[cfg(all(target_arch = "x86_64", not(test)))]
use crate::user_copy::errno::EFAULT;
use crate::user_copy::validate::bounds::check_user_bounds;
#[cfg(all(target_arch = "x86_64", not(test)))]
use crate::user_copy::validate::fault::try_fault_user_page;

/// Validates that a user memory buffer is completely contained within valid user virtual address space.
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
