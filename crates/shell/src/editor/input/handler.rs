// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Keypress dispatcher and modal dialog handler for the editor.

use keira_io::vga;

use super::action::{cut_line, delete_backspace, insert_char, insert_newline, paste_line};
use super::cursor::{cursor_down, cursor_left, cursor_right, cursor_up};
use crate::editor::buffer::file::{editor_save_file, editor_start};
use crate::editor::buffer::status::{get_file_last_line, set_status_cur_pos, set_status_msg_wrote};
use crate::editor::render::metrics::{canvas_rows, content_cols};
use crate::editor::render::view::editor_redraw;
use crate::state::session::{
    EDITOR_CONFIRM_EXIT, EDITOR_CONFIRM_SAVE, EDITOR_GRID, EDITOR_HELP_MODE, EDITOR_MODIFIED,
    EDITOR_STATUS_COLOR, EDITOR_STATUS_LEN, EDITOR_STATUS_MSG, EDIT_CUR_X, EDIT_CUR_Y,
    EDIT_FILENAME, EDIT_FILENAME_LEN, EDIT_SCROLL_Y, IN_EDITOR_MODE, IN_SEARCH_MODE, LINE_LENS,
    SEARCH_BUFFER, SEARCH_LEN,
};

pub const KEY_UP: u8 = 0x80;
pub const KEY_DOWN: u8 = 0x81;
pub const KEY_LEFT: u8 = 0x82;
pub const KEY_RIGHT: u8 = 0x83;
pub const KEY_F3: u8 = 0x84;
pub const KEY_F10: u8 = 0x85;

/// Dispatch and execute keypress event inside the interactive text editor.
pub unsafe fn editor_handle_keypress(c: u8) {
    let c_rows = canvas_rows() as u16;
    let _c_cols = content_cols();
    let last_line = get_file_last_line() as u16;

    if EDITOR_HELP_MODE {
        EDITOR_HELP_MODE = false;
        editor_redraw();
        return;
    }

    if EDITOR_CONFIRM_SAVE {
        match c {
            b'y' | b'Y' => {
                if let Err(e) = editor_save_file() {
                    vga::init();
                    vga::set_color(vga::Color::LightRed, vga::Color::Black);
                    vga::print_str("Error saving file: ");
                    vga::print_str(e);
                    vga::print_str("\nPress any key to return...\n");
                    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                    EDITOR_CONFIRM_SAVE = false;
                    EDITOR_CONFIRM_EXIT = true;
                    return;
                }
                IN_EDITOR_MODE = false;
                vga::init();
                crate::print_prompt();
            }
            b'n' | b'N' => {
                IN_EDITOR_MODE = false;
                vga::init();
                crate::print_prompt();
            }
            b'c' | b'C' | 27 => {
                EDITOR_CONFIRM_SAVE = false;
                let msg = b"[ Cancelled ]";
                EDITOR_STATUS_LEN = msg.len();
                EDITOR_STATUS_MSG[..msg.len()].copy_from_slice(msg);
                EDITOR_STATUS_COLOR = vga::Color::White;
                editor_redraw();
            }
            _ => {}
        }
        return;
    }

    if EDITOR_CONFIRM_EXIT {
        IN_EDITOR_MODE = false;
        vga::init();
        crate::print_prompt();
        return;
    }

    if IN_SEARCH_MODE {
        match c {
            27 => {
                IN_SEARCH_MODE = false;
                SEARCH_LEN = 0;
                SEARCH_BUFFER = [0; 16];
                let msg = b"[ Cancelled ]";
                EDITOR_STATUS_LEN = msg.len();
                EDITOR_STATUS_MSG[..msg.len()].copy_from_slice(msg);
                EDITOR_STATUS_COLOR = vga::Color::White;
                editor_redraw();
            }
            10 | 13 => {
                let mut found = false;
                let term = &SEARCH_BUFFER[..SEARCH_LEN];
                if SEARCH_LEN > 0 {
                    'outer: for y in 0..128 {
                        let len = LINE_LENS[y] as usize;
                        if len >= SEARCH_LEN {
                            for x in 0..=(len - SEARCH_LEN) {
                                let mut matched = true;
                                for i in 0..SEARCH_LEN {
                                    if EDITOR_GRID[y][x + i] != term[i] {
                                        matched = false;
                                        break;
                                    }
                                }
                                if matched {
                                    EDIT_CUR_Y = y as u16;
                                    EDIT_CUR_X = x as u16;
                                    if EDIT_CUR_Y < EDIT_SCROLL_Y {
                                        EDIT_SCROLL_Y = EDIT_CUR_Y;
                                    } else if EDIT_CUR_Y >= EDIT_SCROLL_Y + c_rows {
                                        EDIT_SCROLL_Y = EDIT_CUR_Y.saturating_sub(c_rows - 1);
                                    }
                                    found = true;
                                    let msg = b"[ Match found ]";
                                    EDITOR_STATUS_LEN = msg.len();
                                    EDITOR_STATUS_MSG[..msg.len()].copy_from_slice(msg);
                                    EDITOR_STATUS_COLOR = vga::Color::LightGreen;
                                    break 'outer;
                                }
                            }
                        }
                    }
                }
                if !found && SEARCH_LEN > 0 {
                    let msg = b"[ Search term not found ]";
                    EDITOR_STATUS_LEN = msg.len();
                    EDITOR_STATUS_MSG[..msg.len()].copy_from_slice(msg);
                    EDITOR_STATUS_COLOR = vga::Color::LightRed;
                }
                IN_SEARCH_MODE = false;
                editor_redraw();
            }
            8 => {
                if SEARCH_LEN > 0 {
                    SEARCH_LEN -= 1;
                    SEARCH_BUFFER[SEARCH_LEN] = 0;
                    editor_redraw();
                }
            }
            _ => {
                if SEARCH_LEN < 16 && (32..=126).contains(&c) {
                    SEARCH_BUFFER[SEARCH_LEN] = c;
                    SEARCH_LEN += 1;
                    editor_redraw();
                }
            }
        }
        return;
    }

    match c {
        KEY_UP => cursor_up(),
        KEY_DOWN => cursor_down(last_line),
        KEY_LEFT => cursor_left(),
        KEY_RIGHT => cursor_right(last_line),
        7 => {
            EDITOR_HELP_MODE = true;
            editor_redraw();
        }
        15 | KEY_F3 | 19 => match editor_save_file() {
            Ok(lines) => {
                set_status_msg_wrote(lines);
                editor_redraw();
            }
            Err(_) => {
                let msg = b"[ Error writing file! ]";
                EDITOR_STATUS_LEN = msg.len();
                EDITOR_STATUS_MSG[..msg.len()].copy_from_slice(msg);
                EDITOR_STATUS_COLOR = vga::Color::LightRed;
                editor_redraw();
            }
        },
        24 | KEY_F10 | 17 | 27 => {
            if EDITOR_MODIFIED {
                EDITOR_CONFIRM_SAVE = true;
                editor_redraw();
            } else {
                IN_EDITOR_MODE = false;
                vga::init();
                crate::print_prompt();
            }
        }
        23 | 6 => {
            IN_SEARCH_MODE = true;
            SEARCH_LEN = 0;
            SEARCH_BUFFER = [0; 16];
            editor_redraw();
        }
        11 => cut_line(),
        21 => paste_line(),
        3 => {
            set_status_cur_pos();
            editor_redraw();
        }
        18 => {
            let filename_slice = &EDIT_FILENAME[..EDIT_FILENAME_LEN];
            if let Ok(filename) = core::str::from_utf8(filename_slice) {
                let _ = editor_start(filename);
            }
        }
        10 | 13 => insert_newline(),
        8 => delete_backspace(),
        _ => {
            if (32..=126).contains(&c) {
                insert_char(c);
            }
        }
    }
}
