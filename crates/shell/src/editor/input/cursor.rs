// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Cursor navigation and scrolling handlers for editor.

use crate::editor::render::metrics::{canvas_rows, content_cols};
use crate::editor::render::view::editor_redraw;
use crate::state::session::{EDIT_CUR_X, EDIT_CUR_Y, EDIT_SCROLL_Y, LINE_LENS};

/// Move cursor up within document bounds and update view scroll offset.
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
