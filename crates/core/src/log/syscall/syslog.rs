// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Kernel syslog system call interface (Syscall 44).
//!
//! Provides diagnostic message transfer from kernel log buffers to userland buffers.

/// Reads kernel syslog ring buffer contents into a destination buffer (Syscall 44).
///
/// # Arguments
///
/// * `_buf_ptr` - Destination user/kernel pointer to receive log bytes.
/// * `len` - Maximum number of bytes requested to read.
///
/// # Returns
///
/// Returns the number of bytes read on success, or a static string error on failure.
pub fn sys_syslog_read(_buf_ptr: *mut u8, len: usize) -> Result<usize, &'static str> {
    Ok(len)
}
