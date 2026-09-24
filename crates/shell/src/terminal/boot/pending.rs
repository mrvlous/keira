// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Event loop processor for pending commands, authentication, and background services.

use keira_io::vga;

use crate::executor::dispatch::entry::execute_command;
use crate::history::storage::history_push;
use crate::service::daemon::tick_all;
use crate::state::session::{
    BUFFER_LEN, BUFFER_SIZE, COMMAND_READY, CURRENT_USER, CURRENT_USER_LEN, HISTORY_INDEX,
    INPUT_BUFFER, IN_EDITOR_MODE, IN_LOGIN_MODE, IN_PLEASE_MODE, IS_ADMIN, LOGIN_ATTEMPTS,
    LOGIN_USERNAME, LOGIN_USERNAME_LEN, PLEASE_ATTEMPTS, PLEASE_COMMAND, PLEASE_COMMAND_LEN,
    SHELL_PATH, SHELL_PATH_LEN,
};
use crate::terminal::prompt::display::print_prompt;

/// Process any pending shell commands and background service ticks.
pub fn process_pending() {
    unsafe {
        tick_all();
        keira_task::scheduler::reap_orphaned_zombies();

        if !COMMAND_READY {
            return;
        }

        if IN_PLEASE_MODE {
            COMMAND_READY = false;

            let password_slice = &INPUT_BUFFER[..BUFFER_LEN];
            let user_str = core::str::from_utf8(&CURRENT_USER[..CURRENT_USER_LEN]).unwrap_or("");
            let (found, pwd, pwd_len) = crate::cmds::user::lookup_user(user_str);
            let is_correct = if user_str == "admin" {
                true
            } else if found && pwd_len > 0 {
                password_slice == &pwd[..pwd_len]
            } else {
                password_slice == b"keira"
            };

            BUFFER_LEN = 0;
            INPUT_BUFFER = [0u8; BUFFER_SIZE];

            if is_correct {
                PLEASE_ATTEMPTS = 0;
                IN_PLEASE_MODE = false;
                IS_ADMIN = true;
                if let Ok(cmd_str) = core::str::from_utf8(&PLEASE_COMMAND[..PLEASE_COMMAND_LEN]) {
                    execute_command(cmd_str);
                }
                IS_ADMIN = false;
                PLEASE_COMMAND = [0u8; 128];
                PLEASE_COMMAND_LEN = 0;

                if !IN_PLEASE_MODE && !IN_LOGIN_MODE {
                    print_prompt();
                }
            } else {
                PLEASE_ATTEMPTS += 1;
                if PLEASE_ATTEMPTS < 3 {
                    vga::set_color(vga::Color::LightRed, vga::Color::Black);
                    vga::print_str("please: incorrect password (attempt ");
                    vga::print_u64(PLEASE_ATTEMPTS as u64);
                    vga::print_str("/3). Try again: ");
                    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                } else {
                    vga::set_color(vga::Color::LightRed, vga::Color::Black);
                    vga::print_str("please: 3 incorrect password attempts. Aborted.\n");
                    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                    PLEASE_ATTEMPTS = 0;
                    IN_PLEASE_MODE = false;
                    PLEASE_COMMAND = [0u8; 128];
                    PLEASE_COMMAND_LEN = 0;
                    print_prompt();
                }
            }
            return;
        }

        if IN_LOGIN_MODE {
            COMMAND_READY = false;

            let password_slice = &INPUT_BUFFER[..BUFFER_LEN];
            let login_user_str =
                core::str::from_utf8(&LOGIN_USERNAME[..LOGIN_USERNAME_LEN]).unwrap_or("");
            let (found, pwd, pwd_len) = crate::cmds::user::lookup_user(login_user_str);
            let is_correct = if login_user_str == "admin" {
                if found && pwd_len > 0 {
                    password_slice == &pwd[..pwd_len]
                } else {
                    password_slice == b"keira"
                }
            } else {
                found && password_slice == &pwd[..pwd_len]
            };

            BUFFER_LEN = 0;
            INPUT_BUFFER = [0u8; BUFFER_SIZE];

            if is_correct {
                LOGIN_ATTEMPTS = 0;
                IN_LOGIN_MODE = false;

                CURRENT_USER = [0u8; 16];
                CURRENT_USER[..login_user_str.len()].copy_from_slice(login_user_str.as_bytes());
                CURRENT_USER_LEN = login_user_str.len();

                IS_ADMIN = login_user_str == "admin";

                vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                vga::print_str("Successfully logged in as ");
                vga::print_str(login_user_str);
                vga::print_str(".\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);

                let mut home_buf = [0u8; 32];
                let prefix = b"/users/";
                home_buf[..prefix.len()].copy_from_slice(prefix);
                home_buf[prefix.len()..prefix.len() + login_user_str.len()]
                    .copy_from_slice(login_user_str.as_bytes());
                let home_str =
                    core::str::from_utf8(&home_buf[..prefix.len() + login_user_str.len()])
                        .unwrap_or("/users/admin");

                let _ = keira_fs::fat::change_directory(home_str);
                let rel_path = &home_str[1..];
                SHELL_PATH = [0u8; 80];
                SHELL_PATH[..rel_path.len()].copy_from_slice(rel_path.as_bytes());
                SHELL_PATH_LEN = rel_path.len();

                LOGIN_USERNAME = [0u8; 16];
                LOGIN_USERNAME_LEN = 0;

                print_prompt();
            } else {
                LOGIN_ATTEMPTS += 1;
                if LOGIN_ATTEMPTS < 3 {
                    vga::set_color(vga::Color::LightRed, vga::Color::Black);
                    vga::print_str("login: incorrect password (attempt ");
                    vga::print_u64(LOGIN_ATTEMPTS as u64);
                    vga::print_str("/3). Password for ");
                    vga::print_str(login_user_str);
                    vga::print_str(": ");
                    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                } else {
                    vga::set_color(vga::Color::LightRed, vga::Color::Black);
                    vga::print_str("login: 3 incorrect password attempts. Access denied.\n");
                    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                    LOGIN_ATTEMPTS = 0;
                    IN_LOGIN_MODE = false;
                    LOGIN_USERNAME = [0u8; 16];
                    LOGIN_USERNAME_LEN = 0;
                    print_prompt();
                }
            }
            return;
        }

        history_push();
        HISTORY_INDEX = -1;

        let buffer_slice = &INPUT_BUFFER[..BUFFER_LEN];
        if let Ok(cmd_str) = core::str::from_utf8(buffer_slice) {
            let trimmed = cmd_str.trim();
            if !trimmed.is_empty() {
                execute_command(trimmed);
            }
        } else {
            vga::print_str("Error: invalid input encoding\n");
        }

        BUFFER_LEN = 0;
        COMMAND_READY = false;

        if !IN_PLEASE_MODE && !IN_LOGIN_MODE && !IN_EDITOR_MODE {
            print_prompt();
        }
    }
}
