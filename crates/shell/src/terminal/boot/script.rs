// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Early shell startup initialization and boot script execution.

use crate::state::session::{HOSTNAME, HOSTNAME_LEN, SHELL_PATH, SHELL_PATH_LEN};

const HOSTNAME_PATH: &str = "/etc/hostname";

/// Load hostname from `/etc/hostname` into global state on boot.
pub fn load_hostname() {
    unsafe {
        let mut buf = [0u8; 32];
        if let Ok(len) = keira_fs::fat::read_file_content(HOSTNAME_PATH, &mut buf) {
            let mut actual_len = len;
            while actual_len > 0
                && (buf[actual_len - 1] == b'\n'
                    || buf[actual_len - 1] == b'\r'
                    || buf[actual_len - 1] == b' ')
            {
                actual_len -= 1;
            }

            if actual_len > 0 && actual_len <= 32 {
                HOSTNAME = [b' '; 32];
                HOSTNAME[..actual_len].copy_from_slice(&buf[..actual_len]);
                HOSTNAME_LEN = actual_len;
            }
        }
    }
}

/// Execute initial environment setup and configure default working directory.
pub fn run_boot_script() {
    unsafe {
        load_hostname();

        let _ = keira_fs::fat::change_directory("/");
        SHELL_PATH = [0u8; 80];
        SHELL_PATH_LEN = 0;
    }
}
