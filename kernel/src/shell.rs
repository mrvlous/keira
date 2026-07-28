// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Keira Kernel: Interactive Terminal Shell Module Root
//!
//! Orchestrates the interactive terminal shell, including keypress handling,
//! command execution, history, editing, and autocompletion.

pub mod autocomplete;
pub mod cmds;
pub mod editor;
pub mod executor;
pub mod history;
pub mod state;

// Keyboard scan code translation constants
pub const KEY_UP: u8 = 0x80;
pub const KEY_DOWN: u8 = 0x81;
pub const KEY_LEFT: u8 = 0x82;
pub const KEY_RIGHT: u8 = 0x83;
pub const KEY_F3: u8 = 0x84;
pub const KEY_F10: u8 = 0x85;

use crate::io::vga;
use autocomplete::handle_autocomplete;
use editor::editor_handle_keypress;
use executor::execute_command;
use history::{history_load, history_push};
use state::*;

/// Print the Keira ASCII Logo
pub fn print_logo() {
    unsafe {
        vga::set_color(CURRENT_THEME.text_fg, CURRENT_THEME.text_bg);
    }
    vga::print_str("Keira Kernel ");
    vga::print_str(env!("CARGO_PKG_VERSION"));
    vga::print_str("-keira-1 (tty1)\n\n");
}

/// Print the shell prompt and record cursor position
///
/// # Safety
/// This function reads from global mutable state `CURRENT_USER` and `CURRENT_USER_LEN`.
pub unsafe fn get_current_user_home() -> &'static str {
    match core::str::from_utf8(&CURRENT_USER[..CURRENT_USER_LEN]) {
        Ok("admin") => "users/admin",
        Ok("guest") => "users/guest",
        _ => "users/default",
    }
}

pub fn print_prompt() {
    unsafe {
        let user_color = if IS_ADMIN {
            vga::Color::LightRed
        } else {
            CURRENT_THEME.user
        };
        vga::set_color(user_color, CURRENT_THEME.text_bg);
        if let Ok(user_str) = core::str::from_utf8(&CURRENT_USER[..CURRENT_USER_LEN]) {
            vga::print_str(user_str);
        } else {
            vga::print_str("default");
        }

        vga::set_color(CURRENT_THEME.host, CURRENT_THEME.text_bg);
        vga::print_str("@keira ");

        vga::set_color(CURRENT_THEME.path, CURRENT_THEME.text_bg);
        let current_path = core::str::from_utf8(&SHELL_PATH[..SHELL_PATH_LEN]).unwrap_or_default();

        let home_path = get_current_user_home();

        if current_path.is_empty() {
            vga::print_str("/ ");
        } else if current_path == home_path {
            // Home directory shorthand: tilde symbol
            vga::putchar(b'~');
            vga::print_str(" ");
        } else if current_path.starts_with(home_path)
            && current_path.len() > home_path.len()
            && current_path.as_bytes()[home_path.len()] == b'/'
        {
            vga::putchar(b'~');
            vga::print_str(&current_path[home_path.len()..]);
            vga::print_str(" ");
        } else {
            vga::print_str("/");
            vga::print_str(current_path);
            vga::print_str(" ");
        }

        vga::set_color(CURRENT_THEME.symbol, CURRENT_THEME.text_bg);
        vga::putchar(b'>');
        vga::print_str(" ");
        vga::set_color(CURRENT_THEME.text_fg, CURRENT_THEME.text_bg);
    }

    // Save cursor position right after prompt for history navigation
    unsafe {
        PROMPT_COL = vga::get_cursor_col();
        PROMPT_ROW = vga::get_cursor_row();
    }
}

pub fn run_boot_script() {
    unsafe {
        if crate::fs::fat::change_directory("/users/default").is_ok() {
            let initial_path = "users/default";
            SHELL_PATH[..initial_path.len()].copy_from_slice(initial_path.as_bytes());
            SHELL_PATH_LEN = initial_path.len();
        }
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
                // Ctrl+C
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
                // Enter
                10 | 13 => {
                    vga::print_str("\n");
                    COMMAND_READY = true;
                }
                // Backspace
                8 => {
                    if BUFFER_LEN > 0 {
                        BUFFER_LEN -= 1;
                        INPUT_BUFFER[BUFFER_LEN] = 0;
                    }
                }
                // Ignore autocomplete and arrows during password input
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
            // Ctrl+C to abort input
            3 => {
                vga::print_str("^C\n");
                BUFFER_LEN = 0;
                INPUT_BUFFER = [0u8; BUFFER_SIZE];
                print_prompt();
            }
            // Ctrl+L to clear screen
            12 => {
                vga::set_color(CURRENT_THEME.text_fg, CURRENT_THEME.text_bg);
                vga::init();
                print_prompt();
                let typed = &INPUT_BUFFER[..BUFFER_LEN];
                if let Ok(s) = core::str::from_utf8(typed) {
                    vga::print_str(s);
                }
            }
            // Tab key (9) for autocomplete
            9 => {
                handle_autocomplete();
            }
            // Backspace
            8 => {
                if BUFFER_LEN > 0 {
                    BUFFER_LEN -= 1;
                    INPUT_BUFFER[BUFFER_LEN] = 0;
                    vga::backspace();
                }
            }
            // Enter
            10 | 13 => {
                vga::print_str("\n");
                COMMAND_READY = true;
            }
            // Arrow Up
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
            // Arrow Down
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
            // Regular characters
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

/// Process any pending shell commands.
pub fn process_pending() {
    unsafe {
        if !COMMAND_READY {
            return;
        }

        if IN_PLEASE_MODE {
            IN_PLEASE_MODE = false;
            COMMAND_READY = false;

            let password_slice = &INPUT_BUFFER[..BUFFER_LEN];
            let is_correct = password_slice == b"keira";

            // Reset buffer
            BUFFER_LEN = 0;
            INPUT_BUFFER = [0u8; BUFFER_SIZE];

            if is_correct {
                IS_ADMIN = true;
                if let Ok(cmd_str) = core::str::from_utf8(&PLEASE_COMMAND[..PLEASE_COMMAND_LEN]) {
                    execute_command(cmd_str);
                }
                IS_ADMIN = false;
            } else {
                vga::set_color(vga::Color::LightRed, CURRENT_THEME.text_bg);
                vga::print_str("please: incorrect password.\n");
                vga::set_color(CURRENT_THEME.text_fg, CURRENT_THEME.text_bg);
            }

            PLEASE_COMMAND = [0u8; 128];
            PLEASE_COMMAND_LEN = 0;

            if !IN_PLEASE_MODE && !IN_LOGIN_MODE {
                print_prompt();
            }
            return;
        }

        if IN_LOGIN_MODE {
            IN_LOGIN_MODE = false;
            COMMAND_READY = false;

            let password_slice = &INPUT_BUFFER[..BUFFER_LEN];
            let is_correct = password_slice == b"keira";

            // Reset buffer
            BUFFER_LEN = 0;
            INPUT_BUFFER = [0u8; BUFFER_SIZE];

            if is_correct {
                if let Ok(user_str) = core::str::from_utf8(&LOGIN_USERNAME[..LOGIN_USERNAME_LEN]) {
                    CURRENT_USER = [0u8; 16];
                    CURRENT_USER[..user_str.len()].copy_from_slice(user_str.as_bytes());
                    CURRENT_USER_LEN = user_str.len();

                    vga::set_color(vga::Color::LightGreen, CURRENT_THEME.text_bg);
                    vga::print_str("Successfully logged in as admin.\n");
                    vga::set_color(CURRENT_THEME.text_fg, CURRENT_THEME.text_bg);

                    // Switch to admin home folder
                    let _ = crate::fs::fat::change_directory("/users/admin");
                    let initial_path = "users/admin";
                    SHELL_PATH = [0u8; 80];
                    SHELL_PATH[..initial_path.len()].copy_from_slice(initial_path.as_bytes());
                    SHELL_PATH_LEN = initial_path.len();
                }
            } else {
                vga::set_color(vga::Color::LightRed, CURRENT_THEME.text_bg);
                vga::print_str("login: incorrect password.\n");
                vga::set_color(CURRENT_THEME.text_fg, CURRENT_THEME.text_bg);
            }

            LOGIN_USERNAME = [0u8; 16];
            LOGIN_USERNAME_LEN = 0;

            if !IN_PLEASE_MODE && !IN_LOGIN_MODE {
                print_prompt();
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

        // Reset buffer
        BUFFER_LEN = 0;
        COMMAND_READY = false;

        if !IN_PLEASE_MODE && !IN_LOGIN_MODE {
            print_prompt();
        }
    }
}
