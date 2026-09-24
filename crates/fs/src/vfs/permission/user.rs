// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Active VFS user context management and identification credentials.

/// Global active VFS user identification name buffer.
pub static mut CURRENT_VFS_USER: [u8; 16] = *b"admin           ";

/// Length in bytes of the active VFS user name string.
pub static mut CURRENT_VFS_USER_LEN: usize = 5;

/// Configures the active user name for VFS access checks.
pub fn set_vfs_user(user: &str) {
    unsafe {
        CURRENT_VFS_USER = [0u8; 16];
        let copy_len = core::cmp::min(user.len(), 16);
        CURRENT_VFS_USER[..copy_len].copy_from_slice(&user.as_bytes()[..copy_len]);
        CURRENT_VFS_USER_LEN = copy_len;
    }
}

/// Retrieves the active user name as an immutable string slice.
pub fn get_vfs_user() -> &'static str {
    unsafe {
        let len = CURRENT_VFS_USER_LEN.min(16);
        match core::str::from_utf8(&CURRENT_VFS_USER[..len]) {
            Ok(s) => s,
            Err(_) => "admin",
        }
    }
}
