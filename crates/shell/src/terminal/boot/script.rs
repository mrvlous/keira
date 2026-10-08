// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Early shell startup initialization and boot script execution.

use crate::state::session::{SHELL_PATH, SHELL_PATH_LEN};

/// Execute initial environment setup and configure default working directory.
pub fn run_boot_script() {
    unsafe {
        crate::cmds::hostname::load_hostname();

        let _ = keira_fs::fat::change_directory("/");
        SHELL_PATH = [0u8; 80];
        SHELL_PATH_LEN = 0;
    }
}
