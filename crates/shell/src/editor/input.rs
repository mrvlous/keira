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

//! Subsystem module: input.

pub mod action {
use keira_io::vga;

use crate::editor::buffer::status::get_file_last_line;
use crate::editor::render::metrics::{canvas_rows, content_cols, GRID_LINE_WIDTH};
use crate::editor::render::view::editor_redraw;
use crate::state::session::{
    EDITOR_CUT_BUFFER, EDITOR_CUT_LEN, EDITOR_GRID, EDITOR_HAS_CUT, EDITOR_MODIFIED,
    EDITOR_STATUS_COLOR, EDITOR_STATUS_LEN, EDITOR_STATUS_MSG, EDIT_CUR_X, EDIT_CUR_Y,
    EDIT_SCROLL_Y, LINE_LENS,
};

/// Insert a printable character at current cursor position.
///
/// # Safety
/// Directly manipulates global editor text grid and cursor positions.
pub unsafe fn insert_char(c: u8) {
    let c_cols = content_cols();
    let cur_y = EDIT_CUR_Y as usize;
    let cur_x = EDIT_CUR_X as usize;
    let len = LINE_LENS[cur_y] as usize;

    if len < c_cols && cur_x < c_cols {
        for x in (cur_x..len).rev() {
            EDITOR_GRID[cur_y][x + 1] = EDITOR_GRID[cur_y][x];
        }
        EDITOR_GRID[cur_y][cur_x] = c;
        LINE_LENS[cur_y] += 1;
        EDIT_CUR_X += 1;
        EDITOR_MODIFIED = true;
        editor_redraw();
    }
}

/// Insert a newline, splitting the current line at cursor into two lines.
///
/// # Safety
/// Directly modifies global line grids, length metadata and scroll offsets.
pub unsafe fn insert_newline() {
    let c_rows = canvas_rows() as u16;
    let cur_y = EDIT_CUR_Y as usize;
    let cur_x = EDIT_CUR_X as usize;

    if cur_y < 127 {
        for y in (cur_y + 1..127).rev() {
            EDITOR_GRID[y + 1] = EDITOR_GRID[y];
            LINE_LENS[y + 1] = LINE_LENS[y];
        }

        let old_len = LINE_LENS[cur_y] as usize;
        let new_len = old_len.saturating_sub(cur_x);

        EDITOR_GRID[cur_y + 1] = [b' '; 256];
        if new_len > 0 {
            EDITOR_GRID[cur_y + 1][..new_len].copy_from_slice(&EDITOR_GRID[cur_y][cur_x..old_len]);
        }
        LINE_LENS[cur_y + 1] = new_len as u16;

        EDITOR_GRID[cur_y][cur_x..GRID_LINE_WIDTH].fill(b' ');
        LINE_LENS[cur_y] = cur_x as u16;

        EDIT_CUR_Y += 1;
        EDIT_CUR_X = 0;

        if EDIT_CUR_Y >= EDIT_SCROLL_Y + c_rows {
            EDIT_SCROLL_Y = EDIT_CUR_Y.saturating_sub(c_rows - 1);
        }

        EDITOR_MODIFIED = true;
        editor_redraw();
    }
}

/// Delete character before cursor or merge current line with previous line.
///
/// # Safety
/// Directly modifies editor text cells, shifts buffer lines and redraws terminal.
pub unsafe fn delete_backspace() {
    let c_cols = content_cols();
    let cur_y = EDIT_CUR_Y as usize;
    let cur_x = EDIT_CUR_X as usize;

    if cur_x > 0 {
        let len = LINE_LENS[cur_y] as usize;
        for x in (cur_x - 1)..(len.saturating_sub(1)) {
            EDITOR_GRID[cur_y][x] = EDITOR_GRID[cur_y][x + 1];
        }
        if len > 0 {
            EDITOR_GRID[cur_y][len - 1] = b' ';
            LINE_LENS[cur_y] -= 1;
        }
        EDIT_CUR_X -= 1;
        EDITOR_MODIFIED = true;
        editor_redraw();
    } else if cur_y > 0 {
        let prev_len = LINE_LENS[cur_y - 1] as usize;
        let cur_len = LINE_LENS[cur_y] as usize;
        let available = c_cols.saturating_sub(prev_len);
        let to_copy = core::cmp::min(cur_len, available);

        if to_copy > 0 {
            EDITOR_GRID[cur_y - 1][prev_len..prev_len + to_copy]
                .copy_from_slice(&EDITOR_GRID[cur_y][..to_copy]);
            LINE_LENS[cur_y - 1] += to_copy as u16;
        }

        for y in cur_y..127 {
            EDITOR_GRID[y] = EDITOR_GRID[y + 1];
            LINE_LENS[y] = LINE_LENS[y + 1];
        }
        EDITOR_GRID[127] = [b' '; 256];
        LINE_LENS[127] = 0;

        EDIT_CUR_Y -= 1;
        EDIT_CUR_X = prev_len as u16;

        if EDIT_CUR_Y < EDIT_SCROLL_Y {
            EDIT_SCROLL_Y = EDIT_CUR_Y;
        }

        EDITOR_MODIFIED = true;
        editor_redraw();
    }
}

/// Cut the active line into the clipboard buffer.
///
/// # Safety
/// Accesses and copies static mutable editor cut buffer and shifts row lines.
pub unsafe fn cut_line() {
    let y = EDIT_CUR_Y as usize;
    let len = LINE_LENS[y] as usize;
    EDITOR_CUT_BUFFER[..len].copy_from_slice(&EDITOR_GRID[y][..len]);
    EDITOR_CUT_LEN = len as u16;
    EDITOR_HAS_CUT = true;

    for row in y..127 {
        EDITOR_GRID[row] = EDITOR_GRID[row + 1];
        LINE_LENS[row] = LINE_LENS[row + 1];
    }
    EDITOR_GRID[127] = [b' '; 256];
    LINE_LENS[127] = 0;

    if EDIT_CUR_Y > 0 && EDIT_CUR_Y as usize > get_file_last_line() {
        EDIT_CUR_Y = get_file_last_line() as u16;
    }
    EDIT_CUR_X = EDIT_CUR_X.min(LINE_LENS[EDIT_CUR_Y as usize]);

    EDITOR_MODIFIED = true;
    let msg = b"[ Cut 1 line to clipboard ]";
    EDITOR_STATUS_LEN = msg.len();
    EDITOR_STATUS_MSG[..msg.len()].copy_from_slice(msg);
    EDITOR_STATUS_COLOR = vga::Color::LightGreen;
    editor_redraw();
}

/// Paste the line from clipboard buffer into the active line position.
///
/// # Safety
/// Reads static mutable editor clipboard buffer and shifts active editor grid lines.
pub unsafe fn paste_line() {
    if EDITOR_HAS_CUT {
        let y = EDIT_CUR_Y as usize;
        for row in (y + 1..128).rev() {
            EDITOR_GRID[row] = EDITOR_GRID[row - 1];
            LINE_LENS[row] = LINE_LENS[row - 1];
        }
        let len = (EDITOR_CUT_LEN as usize).min(GRID_LINE_WIDTH);
        EDITOR_GRID[y] = [b' '; 256];
        EDITOR_GRID[y][..len].copy_from_slice(&EDITOR_CUT_BUFFER[..len]);
        LINE_LENS[y] = len as u16;

        EDITOR_MODIFIED = true;
        let msg = b"[ Pasted 1 line from clipboard ]";
        EDITOR_STATUS_LEN = msg.len();
        EDITOR_STATUS_MSG[..msg.len()].copy_from_slice(msg);
        EDITOR_STATUS_COLOR = vga::Color::LightGreen;
    } else {
        let msg = b"[ Cut buffer is empty ]";
        EDITOR_STATUS_LEN = msg.len();
        EDITOR_STATUS_MSG[..msg.len()].copy_from_slice(msg);
        EDITOR_STATUS_COLOR = vga::Color::Yellow;
    }
    editor_redraw();
}

}

pub use self::action::*;
pub mod cursor {
use crate::editor::render::metrics::{canvas_rows, content_cols};
use crate::editor::render::view::editor_redraw;
use crate::state::session::{EDIT_CUR_X, EDIT_CUR_Y, EDIT_SCROLL_Y, LINE_LENS};

/// Move cursor up within document bounds and update view scroll offset.
///
/// # Safety
/// Modifies static mutable editor cursor and scroll offsets, redrawing the screen.
pub unsafe fn cursor_up() {
    if EDIT_CUR_Y > 0 {
        EDIT_CUR_Y -= 1;
        let len = LINE_LENS[EDIT_CUR_Y as usize];
        if EDIT_CUR_X > len {
            EDIT_CUR_X = len;
        }
        if EDIT_CUR_Y < EDIT_SCROLL_Y {
            EDIT_SCROLL_Y = EDIT_CUR_Y;
        }
        editor_redraw();
    }
}

/// Move cursor down strictly bounded to last line of document.
///
/// # Safety
/// Modifies static mutable editor cursor coordinates and updates view scroll offsets.
pub unsafe fn cursor_down(last_line: u16) {
    let c_rows = canvas_rows() as u16;
    if EDIT_CUR_Y < last_line {
        EDIT_CUR_Y += 1;
        let len = LINE_LENS[EDIT_CUR_Y as usize];
        if EDIT_CUR_X > len {
            EDIT_CUR_X = len;
        }
        if EDIT_CUR_Y >= EDIT_SCROLL_Y + c_rows {
            EDIT_SCROLL_Y = EDIT_CUR_Y.saturating_sub(c_rows - 1);
        }
        editor_redraw();
    }
}

/// Move cursor left; wrap to previous line tail if at column 0.
///
/// # Safety
/// Directly updates static mutable cursor coordinates and initiates editor redraw.
pub unsafe fn cursor_left() {
    if EDIT_CUR_X > 0 {
        EDIT_CUR_X -= 1;
        editor_redraw();
    } else if EDIT_CUR_Y > 0 {
        EDIT_CUR_Y -= 1;
        EDIT_CUR_X = LINE_LENS[EDIT_CUR_Y as usize];
        if EDIT_CUR_Y < EDIT_SCROLL_Y {
            EDIT_SCROLL_Y = EDIT_CUR_Y;
        }
        editor_redraw();
    }
}

/// Move cursor right; wrap to next line head only if not at EOF.
///
/// # Safety
/// Updates static mutable cursor positions across line boundaries.
pub unsafe fn cursor_right(last_line: u16) {
    let c_cols = content_cols();
    let c_rows = canvas_rows() as u16;
    let len = LINE_LENS[EDIT_CUR_Y as usize];

    if EDIT_CUR_X < len && (EDIT_CUR_X as usize) < c_cols.saturating_sub(1) {
        EDIT_CUR_X += 1;
        editor_redraw();
    } else if EDIT_CUR_X >= len && EDIT_CUR_Y < last_line {
        EDIT_CUR_Y += 1;
        EDIT_CUR_X = 0;
        if EDIT_CUR_Y >= EDIT_SCROLL_Y + c_rows {
            EDIT_SCROLL_Y = EDIT_CUR_Y.saturating_sub(c_rows - 1);
        }
        editor_redraw();
    }
}

}

pub use self::cursor::*;
pub mod handler {
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
///
/// # Safety
/// Directly manipulates global editor state, modal flags and terminal buffers.
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
                    vga::set_color(vga::Color::White, vga::Color::Black);
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

}

pub use self::handler::*;
