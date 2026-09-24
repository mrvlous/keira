// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Editing primitives for character insertion, line splits, deletions, cut, and paste.

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

        for x in cur_x..GRID_LINE_WIDTH {
            EDITOR_GRID[cur_y][x] = b' ';
        }
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
