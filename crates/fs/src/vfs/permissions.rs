// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! POSIX-like user context security and filesystem access control.

use super::path::resolve_alias_path;

pub static mut CURRENT_VFS_USER: [u8; 16] = *b"admin           ";
pub static mut CURRENT_VFS_USER_LEN: usize = 5;

/// Set the active user name for VFS access checks.
pub fn set_vfs_user(user: &str) {
    unsafe {
        CURRENT_VFS_USER = [0u8; 16];
        let copy_len = core::cmp::min(user.len(), 16);
        CURRENT_VFS_USER[..copy_len].copy_from_slice(&user.as_bytes()[..copy_len]);
        CURRENT_VFS_USER_LEN = copy_len;
    }
}

/// Get the active user name as string slice.
pub fn get_vfs_user() -> &'static str {
    unsafe { core::str::from_utf8(&CURRENT_VFS_USER[..CURRENT_VFS_USER_LEN]).unwrap_or("admin") }
}

/// Check access permissions based on active user context and path.
pub fn check_access_permission(path: &str, is_write: bool) -> Result<(), &'static str> {
    let current_user = get_vfs_user();
    if current_user == "admin" {
        return Ok(());
    }

    let clean = resolve_alias_path(path);
    if let Some(rest) = clean.strip_prefix("/users/") {
        let owner = if let Some(slash_idx) = rest.find('/') {
            &rest[..slash_idx]
        } else {
            rest
        };
        if !owner.is_empty() && owner != current_user {
            return Err("Permission denied: Target path belongs to another user");
        }
    } else if clean.starts_with("/system/") && is_write {
        return Err("Permission denied: Only admin can modify system configuration files");
    }
    Ok(())
}
