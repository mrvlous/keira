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

//! Kernel syslog system call interface (Syscall 44).
//!
//! Provides diagnostic message transfer from kernel log buffers to userland buffers.

use crate::log::buffer::ring::klog_read_all;

/// Reads kernel syslog ring buffer contents into a destination buffer (Syscall 44).
///
/// # Arguments
///
/// * `buf_ptr` - Destination user/kernel pointer to receive log bytes.
/// * `len` - Maximum number of bytes requested to read.
///
/// # Safety
///
/// Callers must ensure that `buf_ptr` points to valid, writable memory spanning at least `len` bytes,
/// or that appropriate memory bounds and access permissions have been verified.
///
/// # Returns
///
/// Returns the number of bytes read on success or a static string error on failure.
pub unsafe fn sys_syslog_read(buf_ptr: *mut u8, len: usize) -> Result<usize, &'static str> {
    if buf_ptr.is_null() && len > 0 {
        return Err("Null buffer pointer");
    }
    if len == 0 {
        return Ok(0);
    }
    let slice = core::slice::from_raw_parts_mut(buf_ptr, len);
    let read_bytes = klog_read_all(slice);
    Ok(read_bytes)
}


pub mod syslog {
    pub use super::*;
}
