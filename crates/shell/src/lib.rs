// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

#![no_std]

//! Interactive system shell, text editor, tab auto-completion, and native commands.

pub mod args;
pub mod autocomplete;
pub mod cmds;
pub mod editor;
pub mod executor;
pub mod history;
pub mod service;
pub mod state;

pub use args::CliArgs;

pub const KEY_UP: u8 = 0x80;
pub const KEY_DOWN: u8 = 0x81;
pub const KEY_LEFT: u8 = 0x82;
pub const KEY_RIGHT: u8 = 0x83;
pub const KEY_F3: u8 = 0x84;
pub const KEY_F10: u8 = 0x85;

use keira_io::vga;
use state::*;

pub use autocomplete::handle_autocomplete;
pub use cmds::*;
pub use editor::kvi::editor_handle_keypress;
pub use editor::*;
pub use executor::*;
pub use executor::{execute_command, get_current_user_home};
pub use history::{history_load, history_push};
pub use state::*;

/// Print the Keira ASCII Logo.
pub fn print_logo() {
    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    vga::print_str("Keira Kernel ");
    vga::print_str(env!("CARGO_PKG_VERSION"));
    vga::print_str("-keira-1 (tty1)\n\n");
}

/// Print the shell prompt and record cursor position in authentic Linux console style.
pub fn print_prompt() {
    unsafe {
        let in_ed = &raw const IN_EDITOR_MODE;
        if *in_ed {
            return;
        }

        vga::set_color(vga::Color::White, vga::Color::Black);
        let ulen = core::cmp::min(CURRENT_USER_LEN, 16);
        if let Ok(user_str) = core::str::from_utf8(&CURRENT_USER[..ulen]) {
            vga::print_str(user_str);
        } else {
            vga::print_str("default");
        }

        vga::print_str("@");
        let hlen = core::cmp::min(HOSTNAME_LEN, 32);
        if let Ok(hostname_str) = core::str::from_utf8(&HOSTNAME[..hlen]) {
            vga::print_str(hostname_str);
        } else {
            vga::print_str("keira");
        }
        vga::print_str(":");

        let plen = core::cmp::min(SHELL_PATH_LEN, 80);
        let current_path = core::str::from_utf8(&SHELL_PATH[..plen]).unwrap_or_default();
        let home_path = get_current_user_home();

        if current_path.is_empty() {
            vga::print_str("/");
        } else if current_path == home_path {
            vga::putchar(b'~');
        } else if current_path.starts_with(home_path)
            && current_path.len() > home_path.len()
            && current_path.as_bytes()[home_path.len()] == b'/'
        {
            vga::putchar(b'~');
            vga::print_str(&current_path[home_path.len()..]);
        } else {
            vga::print_str("/");
            vga::print_str(current_path);
        }

        vga::print_str("$ ");
        vga::set_color(vga::Color::White, vga::Color::Black);

        PROMPT_COL = vga::get_cursor_col();
        PROMPT_ROW = vga::get_cursor_row();
    }
}

pub fn run_boot_script() {
    unsafe {
        cmds::hostname::load_hostname();

        CURRENT_USER = [0u8; 16];
        let admin_str = b"admin";
        CURRENT_USER[..admin_str.len()].copy_from_slice(admin_str);
        CURRENT_USER_LEN = admin_str.len();

        if keira_fs::fat::change_directory("/users/admin").is_ok() {
            let initial_path = "users/admin";
            SHELL_PATH[..initial_path.len()].copy_from_slice(initial_path.as_bytes());
            SHELL_PATH_LEN = initial_path.len();
        }

        service::auto_start_enabled_services();
    }
}

/// Handle a keypress from the C keyboard driver.
#[no_mangle]
pub extern "C" fn shell_handle_keypress(c: u8) {
    unsafe {
        if IN_EDITOR_MODE {
            editor_handle_keypress(c);
            return;
        }

        if IN_PLEASE_MODE || IN_LOGIN_MODE {
            match c {
                3 => {
                    vga::print_str("^C\n");
                    BUFFER_LEN = 0;
                    INPUT_BUFFER = [0u8; BUFFER_SIZE];
                    IN_PLEASE_MODE = false;
                    IN_LOGIN_MODE = false;
                    PLEASE_COMMAND = [0u8; 128];
                    PLEASE_COMMAND_LEN = 0;
                    LOGIN_USERNAME = [0u8; 16];
                    LOGIN_USERNAME_LEN = 0;
                    print_prompt();
                }
                10 | 13 => {
                    vga::print_str("\n");
                    COMMAND_READY = true;
                }
                8 => {
                    if BUFFER_LEN > 0 {
                        BUFFER_LEN -= 1;
                        INPUT_BUFFER[BUFFER_LEN] = 0;
                    }
                }
                9 | KEY_UP | KEY_DOWN => {}
                _ => {
                    if BUFFER_LEN < BUFFER_SIZE - 1 {
                        INPUT_BUFFER[BUFFER_LEN] = c;
                        BUFFER_LEN += 1;
                    }
                }
            }
            return;
        }

        match c {
            3 => {
                if let Some(fg_pid) = keira_task::signal::get_foreground_job_pid() {
                    vga::print_str("^C\n");
                    let _ = keira_task::signal::sys_kill(fg_pid, keira_task::signal::SIGINT);
                    return;
                }
                vga::print_str("^C\n");
                BUFFER_LEN = 0;
                INPUT_BUFFER = [0u8; BUFFER_SIZE];
                print_prompt();
            }
            26 => {
                if let Some(fg_pid) = keira_task::signal::get_foreground_job_pid() {
                    vga::print_str("^Z\n");
                    let _ = keira_task::signal::sys_kill(fg_pid, keira_task::signal::SIGSTOP);
                    return;
                }
            }
            12 => {
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                vga::init();
                print_prompt();
                let typed = &INPUT_BUFFER[..BUFFER_LEN];
                if let Ok(s) = core::str::from_utf8(typed) {
                    vga::print_str(s);
                }
            }
            9 => {
                handle_autocomplete();
            }
            8 => {
                if BUFFER_LEN > 0 {
                    BUFFER_LEN -= 1;
                    INPUT_BUFFER[BUFFER_LEN] = 0;
                    vga::backspace();
                }
            }
            10 | 13 => {
                vga::print_str("\n");
                COMMAND_READY = true;
            }
            KEY_UP => {
                if HISTORY_COUNT == 0 {
                    return;
                }
                if HISTORY_INDEX < 0 {
                    HISTORY_INDEX = (HISTORY_COUNT as isize) - 1;
                } else if HISTORY_INDEX > 0 {
                    let oldest = if HISTORY_COUNT > HISTORY_SIZE {
                        (HISTORY_COUNT - HISTORY_SIZE) as isize
                    } else {
                        0
                    };
                    if HISTORY_INDEX > oldest {
                        HISTORY_INDEX -= 1;
                    }
                }
                let idx = (HISTORY_INDEX as usize) % HISTORY_SIZE;
                history_load(idx);
            }
            KEY_DOWN => {
                if HISTORY_INDEX < 0 {
                    return;
                }
                if HISTORY_INDEX < (HISTORY_COUNT as isize) - 1 {
                    HISTORY_INDEX += 1;
                    let idx = (HISTORY_INDEX as usize) % HISTORY_SIZE;
                    history_load(idx);
                } else {
                    HISTORY_INDEX = -1;
                    vga::set_cursor_pos(PROMPT_ROW, PROMPT_COL);
                    vga::clear_line_from(PROMPT_COL);
                    BUFFER_LEN = 0;
                }
            }
            _ => {
                if BUFFER_LEN < BUFFER_SIZE - 1 {
                    INPUT_BUFFER[BUFFER_LEN] = c;
                    BUFFER_LEN += 1;

                    let s = [c];
                    if let Ok(c_str) = core::str::from_utf8(&s) {
                        vga::print_str(c_str);
                    }
                }
            }
        }
    }
}

/// Process any pending shell commands and background service ticks.
pub fn process_pending() {
    unsafe {
        service::tick_all();

        if !COMMAND_READY {
            return;
        }

        if IN_PLEASE_MODE {
            COMMAND_READY = false;

            let password_slice = &INPUT_BUFFER[..BUFFER_LEN];
            let user_str = core::str::from_utf8(&CURRENT_USER[..CURRENT_USER_LEN]).unwrap_or("");
            let (found, pwd, pwd_len) = cmds::user::lookup_user(user_str);
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
            let (found, pwd, pwd_len) = cmds::user::lookup_user(login_user_str);
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
