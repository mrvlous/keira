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

//! Subsystem module: boot.

pub mod pending {
    use keira_io::vga;

    use crate::executor::dispatch::entry::execute_command;
    use crate::history::storage::history_push;
    use crate::state::session::{
        BUFFER_LEN, COMMAND_READY, HISTORY_INDEX, INPUT_BUFFER, IN_EDITOR_MODE,
    };
    use crate::terminal::prompt::display::print_prompt;

    /// Process any pending shell commands and reap orphaned zombies.
    pub fn process_pending() {
        unsafe {
            keira_task::scheduler::reap_orphaned_zombies();

            if !COMMAND_READY {
                return;
            }

            history_push();
            HISTORY_INDEX = -1;

            let mut local_cmd_buf = [0u8; 512];
            let copy_len = BUFFER_LEN.min(local_cmd_buf.len());
            local_cmd_buf[..copy_len].copy_from_slice(&INPUT_BUFFER[..copy_len]);

            BUFFER_LEN = 0;
            COMMAND_READY = false;

            keira_io::tty::flush_tty();
            keira_io::ps2::flush_input_queue();

            if let Ok(cmd_str) = core::str::from_utf8(&local_cmd_buf[..copy_len]) {
                let trimmed = cmd_str.trim();
                if !trimmed.is_empty() {
                    execute_command(trimmed);
                }
            } else {
                vga::print_str("Error: invalid input encoding\n");
            }

            keira_io::tty::flush_tty();
            keira_io::ps2::flush_input_queue();

            let in_ed = &raw const IN_EDITOR_MODE;
            if !*in_ed {
                print_prompt();
            }
        }
    }
}

pub use self::pending::*;
pub mod script {
    use crate::state::session::{HOSTNAME, HOSTNAME_LEN, SHELL_PATH, SHELL_PATH_LEN};

    const DEFAULT_HOSTNAME: &[u8] = b"keira";
    const HOSTNAME_PATH: &str = "/etc/hostname";

    /// Initialize kernel hostname from static default and synchronize with disk if present.
    pub fn load_hostname() {
        unsafe {
            // Enforce baseline freestanding kernel default: 'keira'
            if HOSTNAME_LEN == 0 {
                HOSTNAME[..DEFAULT_HOSTNAME.len()].copy_from_slice(DEFAULT_HOSTNAME);
                HOSTNAME_LEN = DEFAULT_HOSTNAME.len();
            }

            let mut buf = [0u8; 32];
            if let Ok(len) = keira_fs::vfs::read_file(HOSTNAME_PATH, &mut buf) {
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
}

pub use self::script::*;
