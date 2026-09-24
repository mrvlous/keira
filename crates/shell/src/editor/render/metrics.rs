// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Dynamic display metrics, layout dimensions, and row indices for the editor view.

use keira_io::vga;

/// Maximum line width stored in EDITOR_GRID.
pub const GRID_LINE_WIDTH: usize = 256;
/// Maximum screen buffer columns.
pub const SCREEN_BUF_COLS: usize = 160;
/// Maximum screen buffer rows.
pub const SCREEN_BUF_ROWS: usize = 64;
/// Maximum file buffer size in bytes.
pub const FILE_BUF_SIZE: usize = 16384;
/// Line number gutter column width.
pub const GUTTER_WIDTH: usize = 5;

/// Get the number of text columns the display can show.
pub fn screen_cols() -> usize {
    let c = vga::get_text_cols() as usize;
    if c == 0 {
        80
    } else {
        c.min(SCREEN_BUF_COLS)
    }
}

/// Get the number of text rows the display can show.
pub fn screen_rows() -> usize {
    let r = vga::get_text_rows() as usize;
    if r == 0 {
        25
    } else {
        r.min(SCREEN_BUF_ROWS)
    }
}

/// Number of rows available for the text editing canvas.
pub fn canvas_rows() -> usize {
    let sr = screen_rows();
    if sr > 4 {
        sr - 4
    } else {
        1
    }
}

/// Number of columns available for text content (screen cols minus gutter).
pub fn content_cols() -> usize {
    let sc = screen_cols();
    if sc > GUTTER_WIDTH {
        sc - GUTTER_WIDTH
    } else {
        1
    }
}

/// Status bar row index.
pub fn status_row() -> usize {
    screen_rows().saturating_sub(3)
}

/// First shortcut row index.
pub fn shortcut_row1() -> usize {
    screen_rows().saturating_sub(2)
}

/// Second shortcut row index.
pub fn shortcut_row2() -> usize {
    screen_rows().saturating_sub(1)
}
