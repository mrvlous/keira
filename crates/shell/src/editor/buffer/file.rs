// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Editor file persistence, initialization, and buffer loading.

use keira_io::vga;

use super::status::{get_file_last_line, set_status_msg_read};
use crate::editor::render::metrics::{content_cols, FILE_BUF_SIZE, GRID_LINE_WIDTH};
use crate::editor::render::view::editor_redraw;
use crate::state::session::{
    EDITOR_CONFIRM_EXIT, EDITOR_CONFIRM_SAVE, EDITOR_FILE_BUF, EDITOR_GRID, EDITOR_HELP_MODE,
    EDITOR_MODIFIED, EDITOR_STATUS_COLOR, EDITOR_STATUS_LEN, EDITOR_STATUS_MSG, EDIT_CUR_X,
    EDIT_CUR_Y, EDIT_FILENAME, EDIT_FILENAME_LEN, EDIT_SCROLL_Y, IN_EDITOR_MODE, IN_SEARCH_MODE,
    LINE_LENS, SEARCH_BUFFER, SEARCH_LEN,
};

/// Save current in-memory buffer to persistent FAT16 storage.
pub unsafe fn editor_save_file() -> Result<usize, &'static str> {
    let mut flat_len = 0;
    let last_y = get_file_last_line();

    for y in 0..=last_y {
        let row_len = (LINE_LENS[y] as usize).min(GRID_LINE_WIDTH);
        for x in 0..row_len {
            if flat_len < FILE_BUF_SIZE {
                EDITOR_FILE_BUF[flat_len] = EDITOR_GRID[y][x];
                flat_len += 1;
            }
        }

        if y < last_y && flat_len < FILE_BUF_SIZE {
            EDITOR_FILE_BUF[flat_len] = b'\n';
            flat_len += 1;
        }
    }

    let filename_slice = &EDIT_FILENAME[..EDIT_FILENAME_LEN];
    let filename_str =
        core::str::from_utf8(filename_slice).map_err(|_| "Invalid filename encoding")?;

    keira_fs::vfs::write_file(filename_str, &EDITOR_FILE_BUF[..flat_len])?;
    EDITOR_MODIFIED = false;
    Ok(last_y + 1)
}

/// Start the nano editor session for a given file.
pub unsafe fn editor_start(filename: &str) -> Result<(), &'static str> {
    EDIT_FILENAME = [0; 64];
    EDIT_FILENAME_LEN = core::cmp::min(filename.len(), 64);
    EDIT_FILENAME[..EDIT_FILENAME_LEN].copy_from_slice(filename.as_bytes());

    EDITOR_GRID = [[b' '; 256]; 128];
    LINE_LENS = [0; 128];
    EDIT_CUR_X = 0;
    EDIT_CUR_Y = 0;
    EDIT_SCROLL_Y = 0;
    EDITOR_MODIFIED = false;
    EDITOR_CONFIRM_SAVE = false;
    EDITOR_CONFIRM_EXIT = false;
    EDITOR_HELP_MODE = false;
    IN_SEARCH_MODE = false;
    SEARCH_LEN = 0;
    SEARCH_BUFFER = [0; 16];

    let max_line_w = content_cols().min(GRID_LINE_WIDTH);

    match keira_fs::vfs::read_file(filename, &mut EDITOR_FILE_BUF) {
        Ok(bytes_read) => {
            let mut x = 0;
            let mut y = 0;
            for &b in &EDITOR_FILE_BUF[..bytes_read] {
                if b == b'\n' {
                    if y < 128 {
                        LINE_LENS[y] = x as u16;
                    }
                    x = 0;
                    y += 1;
                    if y >= 128 {
                        break;
                    }
                } else if b == b'\r' {
                    // Skip carriage return
                } else if x < max_line_w && y < 128 {
                    EDITOR_GRID[y][x] = b;
                    x += 1;
                }
            }
            if y < 128 {
                LINE_LENS[y] = x as u16;
            }
            let lines_count = if y >= 128 { 128 } else { y + 1 };
            set_status_msg_read(lines_count);
        }
        Err(_) => {
            let _ = keira_fs::fat::create_file(filename);
            let msg = b"[ New File ]";
            EDITOR_STATUS_LEN = msg.len();
            EDITOR_STATUS_MSG[..msg.len()].copy_from_slice(msg);
            EDITOR_STATUS_COLOR = vga::Color::White;
        }
    }

    IN_EDITOR_MODE = true;
    vga::init();
    editor_redraw();
    Ok(())
}
