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

//! Raw byte slice and typed data copying primitives between kernel memory and user space.

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

/// Safely copies a typed value from kernel memory to a user space virtual address.
///
/// # Safety
/// Caller must ensure that `dest_user_ptr` points to valid user memory capable of holding `T`.
pub unsafe fn copy_val_to_user<T: Copy>(dest_user_ptr: u64, val: &T) -> Result<(), i64> {
    let size = core::mem::size_of::<T>();
    let slice = core::slice::from_raw_parts(val as *const T as *const u8, size);
    copy_to_user(dest_user_ptr, slice)
}

/// Safely copies a typed value from a user space virtual address into kernel memory.
///
/// # Safety
/// Caller must ensure that `src_user_ptr` points to valid user memory initialized with a valid `T`.
pub unsafe fn copy_val_from_user<T: Copy>(src_user_ptr: u64) -> Result<T, i64> {
    let mut val = core::mem::MaybeUninit::<T>::uninit();
    let size = core::mem::size_of::<T>();
    let slice = core::slice::from_raw_parts_mut(val.as_mut_ptr() as *mut u8, size);
    copy_from_user(slice, src_user_ptr)?;
    Ok(val.assume_init())
}
