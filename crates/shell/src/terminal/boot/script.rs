// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Early shell startup initialization and boot script execution.

use crate::service::daemon::auto_start_enabled_services;
use crate::state::session::{CURRENT_USER, CURRENT_USER_LEN, SHELL_PATH, SHELL_PATH_LEN};

/// Execute initial environment setup and start background services.
pub fn run_boot_script() {
    unsafe {
        crate::cmds::hostname::load_hostname();

        CURRENT_USER = [0u8; 16];
        let admin_str = b"admin";
        CURRENT_USER[..admin_str.len()].copy_from_slice(admin_str);
        CURRENT_USER_LEN = admin_str.len();

        if keira_fs::fat::change_directory("/users/admin").is_ok() {
            let initial_path = "users/admin";
            SHELL_PATH[..initial_path.len()].copy_from_slice(initial_path.as_bytes());
            SHELL_PATH_LEN = initial_path.len();
        }

        auto_start_enabled_services();
    }
}
