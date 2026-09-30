// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Raw byte slice copying primitives between kernel memory and user space.

use crate::user_copy::validate::validate_user_ptr;

/// Safely copies data from a kernel buffer to a user space virtual address.
///
/// # Safety
/// Caller must ensure that `dest_user_ptr` points to valid user memory during the transfer.
pub unsafe fn copy_to_user(dest_user_ptr: u64, src: &[u8]) -> Result<(), i64> {
    if src.is_empty() {
        return Ok(());
    }

    validate_user_ptr(dest_user_ptr, src.len() as u64, true)?;
    core::ptr::copy_nonoverlapping(src.as_ptr(), dest_user_ptr as *mut u8, src.len());
    Ok(())
}

/// Safely copies data from a user space virtual address into a kernel buffer.
///
/// # Safety
/// Caller must ensure that `src_user_ptr` points to valid user memory during the transfer.
pub unsafe fn copy_from_user(dest: &mut [u8], src_user_ptr: u64) -> Result<(), i64> {
    if dest.is_empty() {
        return Ok(());
    }

    validate_user_ptr(src_user_ptr, dest.len() as u64, false)?;
    core::ptr::copy_nonoverlapping(src_user_ptr as *const u8, dest.as_mut_ptr(), dest.len());
    Ok(())
}
