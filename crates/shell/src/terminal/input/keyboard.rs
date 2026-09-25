// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Terminal keyboard driver interrupt callback and character buffering.

use keira_io::vga;

use super::keys::{KEY_DOWN, KEY_UP};
use crate::autocomplete::handle_autocomplete;
use crate::editor::editor_handle_keypress;
use crate::history::history_load;
use crate::state::session::{
    BUFFER_LEN, BUFFER_SIZE, COMMAND_READY, HISTORY_COUNT, HISTORY_INDEX, HISTORY_SIZE,
    INPUT_BUFFER, IN_EDITOR_MODE, PROMPT_COL, PROMPT_ROW,
};
use crate::terminal::prompt::display::print_prompt;

/// Handle a keypress from the C keyboard driver.
#[no_mangle]
pub extern "C" fn shell_handle_keypress(c: u8) {
    unsafe {
        if IN_EDITOR_MODE {
            editor_handle_keypress(c);
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
