// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Access permission validation enforcing user space boundaries and system protection.

use super::user::get_vfs_user;
use crate::vfs::path::alias::resolve_alias_path;

/// Validates access rights based on the active user identity and target canonical path.
///
/// Rules enforced:
/// - `admin` possesses unrestricted read and write privileges across all paths.
/// - Unprivileged users cannot modify system configuration or binary files under `/system/`.
/// - Unprivileged users cannot read or modify workspaces owned by other users under `/users/<other>/`.
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
