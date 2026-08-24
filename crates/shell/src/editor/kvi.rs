// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

#![allow(static_mut_refs)]

//! Fullscreen GNU nano-style interactive text editor with syntax highlighting,
//! direct double-buffered rendering, search, clipboard, telemetry, and smooth navigation.
//! All layout dimensions are dynamically computed from the active monitor resolution.

use crate::state::*;
use crate::{KEY_DOWN, KEY_F10, KEY_F3, KEY_LEFT, KEY_RIGHT, KEY_UP};
use keira_io::vga;

/// Maximum line width stored in EDITOR_GRID (must match state.rs).
const GRID_LINE_WIDTH: usize = 256;
/// Maximum screen buffer width (must match state.rs).
const SCREEN_BUF_COLS: usize = 160;
/// Maximum screen buffer height (must match state.rs).
const SCREEN_BUF_ROWS: usize = 64;
/// Maximum file buffer size (must match state.rs).
const FILE_BUF_SIZE: usize = 16384;
/// Gutter width: " 123| " = 5 columns.
const GUTTER_WIDTH: usize = 5;

/// Get the number of text columns the display can show (dynamic per monitor).
fn screen_cols() -> usize {
    let c = vga::get_text_cols() as usize;
    if c == 0 {
        80
    } else {
        c.min(SCREEN_BUF_COLS)
    }
}

/// Get the number of text rows the display can show (dynamic per monitor).
fn screen_rows() -> usize {
    let r = vga::get_text_rows() as usize;
    if r == 0 {
        25
    } else {
        r.min(SCREEN_BUF_ROWS)
    }
}

/// Number of rows available for the text editing canvas.
/// Row 0 = title bar, Row (N-3) = status bar, Rows (N-2, N-1) = shortcuts.
fn canvas_rows() -> usize {
    let sr = screen_rows();
    if sr > 4 {
        sr - 4
    } else {
        1
    }
}

/// Number of columns available for text content (screen cols minus gutter).
fn content_cols() -> usize {
    let sc = screen_cols();
    if sc > GUTTER_WIDTH {
        sc - GUTTER_WIDTH
    } else {
        1
    }
}

/// Status bar row index.
fn status_row() -> usize {
    screen_rows().saturating_sub(3)
}

/// First shortcut row index.
fn shortcut_row1() -> usize {
    screen_rows().saturating_sub(2)
}

/// Second shortcut row index.
fn shortcut_row2() -> usize {
    screen_rows().saturating_sub(1)
}

fn format_u64(mut val: u64, buf: &mut [u8]) -> usize {
    if val == 0 {
        if !buf.is_empty() {
            buf[0] = b'0';
            return 1;
        }
        return 0;
    }
    let mut temp = [0u8; 20];
    let mut len = 0;
    while val > 0 {
        temp[len] = b'0' + (val % 10) as u8;
        len += 1;
        val /= 10;
    }
    let out_len = len.min(buf.len());
    for i in 0..out_len {
        buf[i] = temp[len - 1 - i];
    }
    out_len
}

fn append_bytes(buf: &mut [u8], len: &mut usize, data: &[u8]) {
    let to_copy = data.len().min(buf.len().saturating_sub(*len));
    buf[*len..*len + to_copy].copy_from_slice(&data[..to_copy]);
    *len += to_copy;
}

/// Returns the 0-based index of the last active line in the document.
unsafe fn get_file_last_line() -> usize {
    let mut last = 0;
    for y in 0..128 {
        if LINE_LENS[y] > 0 {
            last = y;
        }
    }
    let cur_y = EDIT_CUR_Y as usize;
    if cur_y > last {
        last = cur_y;
    }
    last
}

/// Returns total line count of the document (at least 1).
unsafe fn get_total_lines() -> usize {
    get_file_last_line() + 1
}

unsafe fn set_status_msg_read(lines: usize) {
    let mut buf = [0u8; 256];
    let mut len = 0;
    append_bytes(&mut buf, &mut len, b"[ Read ");
    len += format_u64(lines as u64, &mut buf[len..]);

    let suffix: &[u8] = if lines == 1 { b" line ]" } else { b" lines ]" };
    append_bytes(&mut buf, &mut len, suffix);

    EDITOR_STATUS_MSG[..len].copy_from_slice(&buf[..len]);
    EDITOR_STATUS_LEN = len;
    EDITOR_STATUS_COLOR = vga::Color::White;
}

unsafe fn set_status_msg_wrote(lines: usize) {
    let mut buf = [0u8; 256];
    let mut len = 0;
    append_bytes(&mut buf, &mut len, b"[ Wrote ");
    len += format_u64(lines as u64, &mut buf[len..]);

    let suffix: &[u8] = if lines == 1 {
        b" line to '"
    } else {
        b" lines to '"
    };
    append_bytes(&mut buf, &mut len, suffix);

    let fname = &EDIT_FILENAME[..EDIT_FILENAME_LEN];
    append_bytes(&mut buf, &mut len, fname);
    append_bytes(&mut buf, &mut len, b"' ]");

    EDITOR_STATUS_MSG[..len].copy_from_slice(&buf[..len]);
    EDITOR_STATUS_LEN = len;
    EDITOR_STATUS_COLOR = vga::Color::LightGreen;
}

unsafe fn set_status_cur_pos() {
    let cur_line = (EDIT_CUR_Y + 1) as usize;
    let total_lines = get_total_lines();
    let mut total_chars = 0;
    let mut char_pos = 0;

    for y in 0..total_lines {
        let llen = LINE_LENS[y] as usize;
        if y < (EDIT_CUR_Y as usize) {
            char_pos += llen + 1;
        } else if y == (EDIT_CUR_Y as usize) {
            char_pos += (EDIT_CUR_X as usize).min(llen);
        }
        total_chars += llen + 1;
    }
    if total_chars > 0 {
        total_chars -= 1;
    }

    let cur_col = (EDIT_CUR_X + 1) as usize;
    let line_len = (LINE_LENS[EDIT_CUR_Y as usize] as usize + 1).max(1);
    let line_pct = (cur_line * 100) / total_lines.max(1);
    let col_pct = (cur_col * 100) / line_len;
    let char_pct = if total_chars > 0 {
        (char_pos * 100) / total_chars
    } else {
        100
    };

    let mut buf = [0u8; 256];
    let mut len = 0;
    append_bytes(&mut buf, &mut len, b"[ line ");
    len += format_u64(cur_line as u64, &mut buf[len..]);
    append_bytes(&mut buf, &mut len, b"/");
    len += format_u64(total_lines as u64, &mut buf[len..]);
    append_bytes(&mut buf, &mut len, b" (");
    len += format_u64(line_pct as u64, &mut buf[len..]);
    append_bytes(&mut buf, &mut len, b"%), col ");
    len += format_u64(cur_col as u64, &mut buf[len..]);
    append_bytes(&mut buf, &mut len, b"/");
    len += format_u64(line_len as u64, &mut buf[len..]);
    append_bytes(&mut buf, &mut len, b" (");
    len += format_u64(col_pct as u64, &mut buf[len..]);
    append_bytes(&mut buf, &mut len, b"%), char ");
    len += format_u64(char_pos as u64, &mut buf[len..]);
    append_bytes(&mut buf, &mut len, b"/");
    len += format_u64(total_chars as u64, &mut buf[len..]);
    append_bytes(&mut buf, &mut len, b" (");
    len += format_u64(char_pct as u64, &mut buf[len..]);
    append_bytes(&mut buf, &mut len, b"%) ]");

    EDITOR_STATUS_MSG[..len].copy_from_slice(&buf[..len]);
    EDITOR_STATUS_LEN = len;
    EDITOR_STATUS_COLOR = vga::Color::White;
}

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
                    // skip CR
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

/// Set a screen buffer cell (bounds-checked against static buffer limits).
unsafe fn set_cell(row: usize, col: usize, ch: u8, fg: vga::Color, bg: vga::Color) {
    if row < SCREEN_BUF_ROWS && col < SCREEN_BUF_COLS {
        EDITOR_SCREEN_CHARS[row][col] = ch;
        EDITOR_SCREEN_FG[row][col] = fg;
        EDITOR_SCREEN_BG[row][col] = bg;
    }
}

/// Redraw the complete fullscreen GNU nano view, dynamically sized to the active display.
pub unsafe fn editor_redraw() {
    let cols = screen_cols();
    let rows = screen_rows();
    let c_rows = canvas_rows();
    let c_cols = content_cols();
    let s_row = status_row();
    let sc_row1 = shortcut_row1();
    let sc_row2 = shortcut_row2();

    // Clear entire screen buffer
    for r in 0..rows {
        for c in 0..cols {
            set_cell(r, c, b' ', vga::Color::LightGrey, vga::Color::Black);
        }
    }

    // 1. Top Title Bar (Row 0)
    for col in 0..cols {
        set_cell(0, col, b' ', vga::Color::White, vga::Color::DarkGrey);
    }

    // Left: "  Keira nano <version>"
    let title_left = b"  Keira nano ";
    let mut t_idx = 0;
    for &b in title_left {
        if t_idx < cols {
            set_cell(0, t_idx, b, vga::Color::White, vga::Color::DarkGrey);
            t_idx += 1;
        }
    }
    let ver_bytes = env!("CARGO_PKG_VERSION").as_bytes();
    for &b in ver_bytes {
        if t_idx < cols {
            set_cell(0, t_idx, b, vga::Color::White, vga::Color::DarkGrey);
            t_idx += 1;
        }
    }

    // Center: "File: <filename>"
    let file_prefix = b"File: ";
    let fname = &EDIT_FILENAME[..EDIT_FILENAME_LEN];
    let file_str_len = file_prefix.len() + fname.len();
    let center_start = (cols / 2).saturating_sub(file_str_len / 2).max(t_idx + 2);
    let mut f_idx = center_start;
    for &b in file_prefix {
        if f_idx < cols {
            set_cell(0, f_idx, b, vga::Color::White, vga::Color::DarkGrey);
            f_idx += 1;
        }
    }
    for &b in fname {
        if f_idx < cols {
            set_cell(0, f_idx, b, vga::Color::White, vga::Color::DarkGrey);
            f_idx += 1;
        }
    }

    // Right: "Modified"
    if EDITOR_MODIFIED {
        let mod_str = b"Modified";
        let start_mod = cols.saturating_sub(mod_str.len() + 2);
        for (i, &b) in mod_str.iter().enumerate() {
            if start_mod + i < cols {
                set_cell(
                    0,
                    start_mod + i,
                    b,
                    vga::Color::Yellow,
                    vga::Color::DarkGrey,
                );
            }
        }
    }

    // 2. Help Modal or Text Canvas
    if EDITOR_HELP_MODE {
        let help_lines: &[&[u8]] = &[
            b"",
            b"  Keira GNU nano Editor - Command Reference Manual",
            b"",
            b"  ^G (Ctrl+G)  Get Help       Display this help manual",
            b"  ^O (Ctrl+O)  WriteOut       Save current buffer to FAT16 file",
            b"  ^W (Ctrl+W)  Where Is       Search for a text string in buffer",
            b"  ^K (Ctrl+K)  Cut Text       Cut current line into clipboard buffer",
            b"  ^U (Ctrl+U)  Paste Text     Paste cut buffer at current cursor line",
            b"  ^C (Ctrl+C)  Cur Pos        Report line, column, and char count",
            b"  ^R (Ctrl+R)  Read File      Reload original file from disk",
            b"  ^X (Ctrl+X)  Exit           Exit editor (prompts to save if modified)",
            b"",
            b"  Arrow Keys                  Move cursor up, down, left, right",
            b"  Enter / Backspace           Insert line breaks / delete & join lines",
            b"",
            b"  [ Press any key or ^G to close this help window ]",
        ];

        for (idx, line) in help_lines.iter().enumerate() {
            let row = idx + 1;
            if row <= c_rows {
                let fg = if idx == 1 || idx == 15 {
                    vga::Color::White
                } else {
                    vga::Color::LightGrey
                };
                for (col, &b) in line.iter().enumerate() {
                    if col < cols {
                        set_cell(row, col, b, fg, vga::Color::Black);
                    }
                }
            }
        }
    } else {
        // 3. Text Buffer Canvas & Line Numbers (Rows 1..=c_rows)
        let total_lines = get_total_lines();

        for y_view in 0..c_rows {
            let actual_y = (EDIT_SCROLL_Y as usize) + y_view;
            let row = y_view + 1;

            if actual_y < total_lines && actual_y < 128 {
                // Render line number gutter
                let line_no = actual_y + 1;
                let mut gutter = [b' '; GUTTER_WIDTH];
                let mut val = line_no;
                let mut num_tmp = [0u8; 4];
                let mut n_len = 0;

                while val > 0 && n_len < 4 {
                    num_tmp[n_len] = b'0' + (val % 10) as u8;
                    n_len += 1;
                    val /= 10;
                }
                let pad = 3usize.saturating_sub(n_len);
                for i in 0..n_len {
                    gutter[pad + i] = num_tmp[n_len - 1 - i];
                }
                gutter[3] = b'|';
                gutter[4] = b' ';

                for g_col in 0..GUTTER_WIDTH.min(cols) {
                    set_cell(
                        row,
                        g_col,
                        gutter[g_col],
                        vga::Color::DarkGrey,
                        vga::Color::Black,
                    );
                }

                // Render line content with syntax highlighting
                let len = core::cmp::min(LINE_LENS[actual_y] as usize, c_cols);
                let mut x = 0;
                let mut highlight_remaining = 0;

                while x < len {
                    // Check search match
                    if SEARCH_LEN > 0 && highlight_remaining == 0 && x + SEARCH_LEN <= len {
                        let mut matched = true;
                        for i in 0..SEARCH_LEN {
                            if EDITOR_GRID[actual_y][x + i] != SEARCH_BUFFER[i] {
                                matched = false;
                                break;
                            }
                        }
                        if matched {
                            highlight_remaining = SEARCH_LEN;
                        }
                    }

                    let bg_color = if highlight_remaining > 0 {
                        vga::Color::Yellow
                    } else {
                        vga::Color::Black
                    };
                    let fg_override = highlight_remaining > 0;
                    let ch = EDITOR_GRID[actual_y][x];

                    // Numbers
                    if (b'0'..=b'9').contains(&ch) {
                        let fg = if fg_override {
                            vga::Color::Black
                        } else {
                            vga::Color::LightRed
                        };
                        let col = GUTTER_WIDTH + x;
                        if col < cols {
                            set_cell(row, col, ch, fg, bg_color);
                        }
                        x += 1;
                        highlight_remaining = highlight_remaining.saturating_sub(1);
                        continue;
                    }

                    // Strings
                    if ch == b'"' || ch == b'\'' {
                        let quote_char = ch;
                        let fg = if fg_override {
                            vga::Color::Black
                        } else {
                            vga::Color::Yellow
                        };
                        let col = GUTTER_WIDTH + x;
                        if col < cols {
                            set_cell(row, col, ch, fg, bg_color);
                        }
                        x += 1;
                        highlight_remaining = highlight_remaining.saturating_sub(1);

                        while x < len {
                            if SEARCH_LEN > 0 && highlight_remaining == 0 && x + SEARCH_LEN <= len {
                                let mut m = true;
                                for i in 0..SEARCH_LEN {
                                    if EDITOR_GRID[actual_y][x + i] != SEARCH_BUFFER[i] {
                                        m = false;
                                        break;
                                    }
                                }
                                if m {
                                    highlight_remaining = SEARCH_LEN;
                                }
                            }
                            let sbg = if highlight_remaining > 0 {
                                vga::Color::Yellow
                            } else {
                                vga::Color::Black
                            };
                            let sfg = if highlight_remaining > 0 {
                                vga::Color::Black
                            } else {
                                vga::Color::Yellow
                            };
                            let sc = EDITOR_GRID[actual_y][x];
                            let scol = GUTTER_WIDTH + x;
                            if scol < cols {
                                set_cell(row, scol, sc, sfg, sbg);
                            }
                            x += 1;
                            highlight_remaining = highlight_remaining.saturating_sub(1);
                            if sc == quote_char {
                                break;
                            }
                        }
                        continue;
                    }

                    // Comments (//)
                    if ch == b'/' && x + 1 < len && EDITOR_GRID[actual_y][x + 1] == b'/' {
                        let comment_fg = if fg_override {
                            vga::Color::Black
                        } else {
                            vga::Color::LightGreen
                        };
                        while x < len {
                            if SEARCH_LEN > 0 && highlight_remaining == 0 && x + SEARCH_LEN <= len {
                                let mut m = true;
                                for i in 0..SEARCH_LEN {
                                    if EDITOR_GRID[actual_y][x + i] != SEARCH_BUFFER[i] {
                                        m = false;
                                        break;
                                    }
                                }
                                if m {
                                    highlight_remaining = SEARCH_LEN;
                                }
                            }
                            let cbg = if highlight_remaining > 0 {
                                vga::Color::Yellow
                            } else {
                                vga::Color::Black
                            };
                            let cfg = if highlight_remaining > 0 {
                                vga::Color::Black
                            } else {
                                comment_fg
                            };
                            let sc = EDITOR_GRID[actual_y][x];
                            let scol = GUTTER_WIDTH + x;
                            if scol < cols {
                                set_cell(row, scol, sc, cfg, cbg);
                            }
                            x += 1;
                            highlight_remaining = highlight_remaining.saturating_sub(1);
                        }
                        continue;
                    }

                    // Operators
                    if matches!(
                        ch,
                        b'=' | b'+'
                            | b'-'
                            | b'*'
                            | b'/'
                            | b'%'
                            | b'&'
                            | b'|'
                            | b'^'
                            | b'!'
                            | b'<'
                            | b'>'
                    ) {
                        let fg = if fg_override {
                            vga::Color::Black
                        } else {
                            vga::Color::White
                        };
                        let col = GUTTER_WIDTH + x;
                        if col < cols {
                            set_cell(row, col, ch, fg, bg_color);
                        }
                        x += 1;
                        highlight_remaining = highlight_remaining.saturating_sub(1);
                        continue;
                    }

                    // Words / Keywords
                    let is_alpha = |b: u8| -> bool {
                        (b'a'..=b'z').contains(&b) || (b'A'..=b'Z').contains(&b)
                    };
                    let is_alnum =
                        |b: u8| -> bool { is_alpha(b) || (b'0'..=b'9').contains(&b) || b == b'_' };

                    if is_alpha(ch) || ch == b'_' {
                        let start = x;
                        while x < len && is_alnum(EDITOR_GRID[actual_y][x]) {
                            x += 1;
                        }
                        let word_slice = &EDITOR_GRID[actual_y][start..x];
                        let is_keyword = matches!(
                            word_slice,
                            b"fn"
                                | b"let"
                                | b"struct"
                                | b"impl"
                                | b"pub"
                                | b"for"
                                | b"if"
                                | b"else"
                                | b"match"
                                | b"return"
                                | b"loop"
                                | b"mut"
                                | b"static"
                                | b"const"
                                | b"use"
                                | b"mod"
                                | b"as"
                                | b"enum"
                                | b"type"
                                | b"true"
                                | b"false"
                                | b"int"
                                | b"char"
                                | b"void"
                                | b"while"
                                | b"include"
                                | b"define"
                        );

                        for (offset, &wb) in word_slice.iter().enumerate() {
                            let word_x = start + offset;
                            if SEARCH_LEN > 0
                                && highlight_remaining == 0
                                && word_x + SEARCH_LEN <= len
                            {
                                let mut m = true;
                                for i in 0..SEARCH_LEN {
                                    if EDITOR_GRID[actual_y][word_x + i] != SEARCH_BUFFER[i] {
                                        m = false;
                                        break;
                                    }
                                }
                                if m {
                                    highlight_remaining = SEARCH_LEN;
                                }
                            }
                            let wbg = if highlight_remaining > 0 {
                                vga::Color::Yellow
                            } else {
                                vga::Color::Black
                            };
                            let wfg = if highlight_remaining > 0 {
                                vga::Color::Black
                            } else if is_keyword {
                                vga::Color::White
                            } else {
                                vga::Color::LightGrey
                            };
                            let col = GUTTER_WIDTH + word_x;
                            if col < cols {
                                set_cell(row, col, wb, wfg, wbg);
                            }
                            highlight_remaining = highlight_remaining.saturating_sub(1);
                        }
                        continue;
                    }

                    // Default character
                    let fg = if fg_override {
                        vga::Color::Black
                    } else {
                        vga::Color::LightGrey
                    };
                    let col = GUTTER_WIDTH + x;
                    if col < cols {
                        set_cell(row, col, ch, fg, bg_color);
                    }
                    x += 1;
                    highlight_remaining = highlight_remaining.saturating_sub(1);
                }
            }
            // Empty lines below EOF are black
        }
    }

    // 4. Status / Prompt Bar (s_row)
    for col in 0..cols {
        set_cell(s_row, col, b' ', vga::Color::White, vga::Color::DarkGrey);
    }

    if EDITOR_CONFIRM_SAVE {
        let msg = b"  Save modified buffer?  (Y)es, (N)o, (C)ancel";
        for (i, &b) in msg.iter().enumerate() {
            if i < cols {
                set_cell(s_row, i, b, vga::Color::White, vga::Color::DarkGrey);
            }
        }
    } else if IN_SEARCH_MODE {
        let pfx = b"  Search: ";
        let mut s_idx = 0;
        for &b in pfx {
            if s_idx < cols {
                set_cell(s_row, s_idx, b, vga::Color::White, vga::Color::DarkGrey);
                s_idx += 1;
            }
        }
        for &b in &SEARCH_BUFFER[..SEARCH_LEN] {
            if s_idx < cols {
                set_cell(s_row, s_idx, b, vga::Color::White, vga::Color::DarkGrey);
                s_idx += 1;
            }
        }
    } else if EDITOR_HELP_MODE {
        let msg = b"  [ Keira nano Help: Press any key to return to editor ]";
        for (i, &b) in msg.iter().enumerate() {
            if i < cols {
                set_cell(s_row, i, b, vga::Color::White, vga::Color::DarkGrey);
            }
        }
    } else if EDITOR_STATUS_LEN > 0 {
        set_cell(s_row, 0, b' ', vga::Color::White, vga::Color::DarkGrey);
        set_cell(s_row, 1, b' ', vga::Color::White, vga::Color::DarkGrey);
        for (i, &b) in EDITOR_STATUS_MSG[..EDITOR_STATUS_LEN].iter().enumerate() {
            if 2 + i < cols {
                set_cell(s_row, 2 + i, b, EDITOR_STATUS_COLOR, vga::Color::DarkGrey);
            }
        }
    }

    // 5. GNU nano 2-Row Shortcut Matrix
    for col in 0..cols {
        set_cell(sc_row1, col, b' ', vga::Color::White, vga::Color::Black);
        set_cell(sc_row2, col, b' ', vga::Color::White, vga::Color::Black);
    }

    // Compute even spacing: 6 shortcuts per row
    let spacing = cols / 6;

    let render_shortcut = |row: usize, idx: usize, key: &[u8], desc: &[u8]| {
        let start = idx * spacing;
        let mut c = start;
        for &b in key {
            if c < cols {
                set_cell(row, c, b, vga::Color::Black, vga::Color::LightGreen);
                c += 1;
            }
        }
        for &b in desc {
            if c < cols {
                set_cell(row, c, b, vga::Color::White, vga::Color::Black);
                c += 1;
            }
        }
    };

    render_shortcut(sc_row1, 0, b"^G", b" Get Help ");
    render_shortcut(sc_row1, 1, b"^O", b" WriteOut ");
    render_shortcut(sc_row1, 2, b"^W", b" Where Is ");
    render_shortcut(sc_row1, 3, b"^K", b" Cut Text ");
    render_shortcut(sc_row1, 4, b"^J", b" Justify  ");
    render_shortcut(sc_row1, 5, b"^C", b" Cur Pos  ");

    render_shortcut(sc_row2, 0, b"^X", b" Exit     ");
    render_shortcut(sc_row2, 1, b"^R", b" Read File");
    render_shortcut(sc_row2, 2, b"^\\", b" Replace  ");
    render_shortcut(sc_row2, 3, b"^U", b" Paste Txt");
    render_shortcut(sc_row2, 4, b"^T", b" To Spell ");
    render_shortcut(sc_row2, 5, b"^_", b" Go To Ln ");

    // 6. Flush screen buffer to display
    for r in 0..rows {
        for c in 0..cols {
            vga::draw_cell(
                r as u16,
                c as u16,
                EDITOR_SCREEN_CHARS[r][c],
                EDITOR_SCREEN_FG[r][c],
                EDITOR_SCREEN_BG[r][c],
            );
        }
    }

    // 7. Position Hardware Cursor
    if EDITOR_CONFIRM_SAVE {
        vga::set_cursor_pos(s_row as u16, 46);
    } else if IN_SEARCH_MODE {
        vga::set_cursor_pos(s_row as u16, (10 + SEARCH_LEN as u16).min(cols as u16 - 1));
    } else if EDITOR_HELP_MODE {
        vga::set_cursor_pos(s_row as u16, 57.min(cols as u16 - 1));
    } else {
        let view_row = (EDIT_CUR_Y.saturating_sub(EDIT_SCROLL_Y) + 1).min(c_rows as u16);
        let view_col = (EDIT_CUR_X + GUTTER_WIDTH as u16).min(cols as u16 - 1);
        vga::set_cursor_pos(view_row, view_col);
    }
}

/// Dispatch and execute keypress event inside GNU nano editor.
pub unsafe fn editor_handle_keypress(c: u8) {
    let c_rows = canvas_rows();
    let c_cols = content_cols();
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
                // Esc: Cancel search
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
                // Enter: Execute search
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
                                    } else if EDIT_CUR_Y >= EDIT_SCROLL_Y + (c_rows as u16) {
                                        EDIT_SCROLL_Y =
                                            EDIT_CUR_Y.saturating_sub((c_rows as u16) - 1);
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
                // Backspace
                if SEARCH_LEN > 0 {
                    SEARCH_LEN -= 1;
                    SEARCH_BUFFER[SEARCH_LEN] = 0;
                    editor_redraw();
                }
            }
            _ => {
                if SEARCH_LEN < 16 && c >= 32 && c <= 126 {
                    SEARCH_BUFFER[SEARCH_LEN] = c;
                    SEARCH_LEN += 1;
                    editor_redraw();
                }
            }
        }
        return;
    }

    // Main keypress dispatch (non-modal)
    match c {
        // Arrow UP: Move up within document bounds
        KEY_UP => {
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

        // Arrow DOWN: Move down, strictly bounded to last line of document (never moves past EOF)
        KEY_DOWN => {
            if EDIT_CUR_Y < last_line {
                EDIT_CUR_Y += 1;
                let len = LINE_LENS[EDIT_CUR_Y as usize];
                if EDIT_CUR_X > len {
                    EDIT_CUR_X = len;
                }
                if EDIT_CUR_Y >= EDIT_SCROLL_Y + (c_rows as u16) {
                    EDIT_SCROLL_Y = EDIT_CUR_Y.saturating_sub((c_rows as u16) - 1);
                }
                editor_redraw();
            }
        }

        // Arrow LEFT: Move left; wrap to previous line tail if at col 0
        KEY_LEFT => {
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

        // Arrow RIGHT: Move right; wrap to next line head only if not at EOF
        KEY_RIGHT => {
            let len = LINE_LENS[EDIT_CUR_Y as usize];
            if EDIT_CUR_X < len && (EDIT_CUR_X as usize) < c_cols.saturating_sub(1) {
                EDIT_CUR_X += 1;
                editor_redraw();
            } else if EDIT_CUR_X >= len && EDIT_CUR_Y < last_line {
                EDIT_CUR_Y += 1;
                EDIT_CUR_X = 0;
                if EDIT_CUR_Y >= EDIT_SCROLL_Y + (c_rows as u16) {
                    EDIT_SCROLL_Y = EDIT_CUR_Y.saturating_sub((c_rows as u16) - 1);
                }
                editor_redraw();
            }
        }

        // ^G (7): Get Help
        7 => {
            EDITOR_HELP_MODE = true;
            editor_redraw();
        }

        // ^O (15) / F3 / Ctrl+S (19): WriteOut / Save
        15 | KEY_F3 | 19 => match editor_save_file() {
            Ok(lines) => {
                set_status_msg_wrote(lines);
                editor_redraw();
            }
            Err(_e) => {
                let msg = b"[ Error writing file! ]";
                EDITOR_STATUS_LEN = msg.len();
                EDITOR_STATUS_MSG[..msg.len()].copy_from_slice(msg);
                EDITOR_STATUS_COLOR = vga::Color::LightRed;
                editor_redraw();
            }
        },

        // ^X (24) / F10 / Ctrl+Q (17) / Esc (27): Exit
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

        // ^W (23) / Ctrl+F (6): Where Is / Search
        23 | 6 => {
            IN_SEARCH_MODE = true;
            SEARCH_LEN = 0;
            SEARCH_BUFFER = [0; 16];
            editor_redraw();
        }

        // ^K (11): Cut Text (Cut current line into clipboard buffer)
        11 => {
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

        // ^U (21): Paste Text (Paste clipboard buffer at current cursor line)
        21 => {
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

        // ^C (3): Cur Pos (Report line, col, char position telemetry)
        3 => {
            set_status_cur_pos();
            editor_redraw();
        }

        // ^R (18): Read File (Reload from disk)
        18 => {
            let filename_slice = &EDIT_FILENAME[..EDIT_FILENAME_LEN];
            if let Ok(filename) = core::str::from_utf8(filename_slice) {
                if let Ok(bytes_read) = keira_fs::vfs::read_file(filename, &mut EDITOR_FILE_BUF) {
                    EDITOR_GRID = [[b' '; 256]; 128];
                    LINE_LENS = [0; 128];
                    let max_line_w = c_cols.min(GRID_LINE_WIDTH);
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
                    EDITOR_MODIFIED = false;
                    editor_redraw();
                }
            }
        }

        // Enter (10 / 13): Insert newline / split line
        10 | 13 => {
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
                    EDITOR_GRID[cur_y + 1][..new_len]
                        .copy_from_slice(&EDITOR_GRID[cur_y][cur_x..old_len]);
                }
                LINE_LENS[cur_y + 1] = new_len as u16;

                for x in cur_x..GRID_LINE_WIDTH {
                    EDITOR_GRID[cur_y][x] = b' ';
                }
                LINE_LENS[cur_y] = cur_x as u16;

                EDIT_CUR_Y += 1;
                EDIT_CUR_X = 0;

                if EDIT_CUR_Y >= EDIT_SCROLL_Y + (c_rows as u16) {
                    EDIT_SCROLL_Y = EDIT_CUR_Y.saturating_sub((c_rows as u16) - 1);
                }

                EDITOR_MODIFIED = true;
                editor_redraw();
            }
        }

        // Backspace (8): Delete char before cursor or join lines at col 0
        8 => {
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

        // Printable Character Insertion (32..=126)
        _ => {
            if (32..=126).contains(&c) {
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
        }
    }
}
