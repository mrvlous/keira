// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! User authorization, home directory lookup, and administrative permissions.

use crate::state::session::{CURRENT_USER, CURRENT_USER_LEN, IS_ADMIN, SHELL_PATH, SHELL_PATH_LEN};

static mut HOME_PATH_BUF: [u8; 32] = [0u8; 32];

/// Returns the home directory path for the currently logged in user.
///
/// # Safety
/// Reads from global mutable user states.
pub unsafe fn get_current_user_home() -> &'static str {
    let mut total_len = 0;
    HOME_PATH_BUF[0..6].copy_from_slice(b"users/");
    total_len += 6;
    let user_len = core::cmp::min(CURRENT_USER_LEN, 16);
    if total_len + user_len <= 32 {
        HOME_PATH_BUF[total_len..total_len + user_len].copy_from_slice(&CURRENT_USER[..user_len]);
        total_len += user_len;
    }
    core::str::from_utf8(&HOME_PATH_BUF[..total_len]).unwrap_or("users/admin")
}

/// Checks whether the current shell execution session is running with admin privileges.
///
/// # Safety
/// Reads from global mutable user states.
pub unsafe fn is_admin_mode() -> bool {
    let ulen = core::cmp::min(CURRENT_USER_LEN, 16);
    IS_ADMIN || matches!(core::str::from_utf8(&CURRENT_USER[..ulen]), Ok("admin"))
}

/// Validates whether the active user has write permissions in the current working directory.
///
/// # Safety
/// Reads from global mutable user and path states.
pub unsafe fn check_write_permission() -> bool {
    if is_admin_mode() {
        return true;
    }
    let current_path = core::str::from_utf8(&SHELL_PATH[..SHELL_PATH_LEN]).unwrap_or_default();
    let home = get_current_user_home();
    current_path == home
        || (current_path.starts_with(home)
            && current_path.len() > home.len()
            && current_path.as_bytes()[home.len()] == b'/')
}
