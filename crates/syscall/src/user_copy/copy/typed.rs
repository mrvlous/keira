// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Typed data copying primitives between kernel memory and user space.

use crate::user_copy::copy::buffer::{copy_from_user, copy_to_user};

/// Safely copies a typed value from kernel memory to a user space virtual address.
pub unsafe fn copy_val_to_user<T: Copy>(dest_user_ptr: u64, val: &T) -> Result<(), i64> {
    let size = core::mem::size_of::<T>();
    let slice = core::slice::from_raw_parts(val as *const T as *const u8, size);
    copy_to_user(dest_user_ptr, slice)
}

/// Safely copies a typed value from a user space virtual address into kernel memory.
pub unsafe fn copy_val_from_user<T: Copy>(src_user_ptr: u64) -> Result<T, i64> {
    let mut val = core::mem::MaybeUninit::<T>::uninit();
    let size = core::mem::size_of::<T>();
    let slice = core::slice::from_raw_parts_mut(val.as_mut_ptr() as *mut u8, size);
    copy_from_user(slice, src_user_ptr)?;
    Ok(val.assume_init())
}
