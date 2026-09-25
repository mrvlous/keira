// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Double-buffered screen rendering for the fullscreen editor canvas.

use keira_io::vga;

use super::metrics::{
    canvas_rows, content_cols, screen_cols, screen_rows, shortcut_row1, shortcut_row2, status_row,
    GUTTER_WIDTH, SCREEN_BUF_COLS, SCREEN_BUF_ROWS,
};
use super::syntax::{is_alnum, is_alpha, is_keyword, is_operator};
use crate::editor::buffer::status::get_total_lines;
use crate::state::session::{
    EDITOR_CONFIRM_SAVE, EDITOR_GRID, EDITOR_HELP_MODE, EDITOR_MODIFIED, EDITOR_SCREEN_BG,
    EDITOR_SCREEN_CHARS, EDITOR_SCREEN_FG, EDITOR_STATUS_COLOR, EDITOR_STATUS_LEN,
    EDITOR_STATUS_MSG, EDIT_CUR_X, EDIT_CUR_Y, EDIT_FILENAME, EDIT_FILENAME_LEN, EDIT_SCROLL_Y,
    IN_SEARCH_MODE, LINE_LENS, SEARCH_BUFFER, SEARCH_LEN,
};

/// Set a screen buffer cell (bounds-checked against static buffer limits).
pub unsafe fn set_cell(row: usize, col: usize, ch: u8, fg: vga::Color, bg: vga::Color) {
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

    // 2. Editor Help Modal Overlay
    if EDITOR_HELP_MODE {
        let help_lines: [&[u8]; 17] = [
            b"  === Keira GNU nano Interactive Text Editor Help ===",
            b"",
            b"  ^G (Ctrl+G)  Get Help       Display this reference screen",
            b"  ^O (Ctrl+O)  WriteOut       Save file changes to FAT16 filesystem",
            b"  ^W (Ctrl+W)  Where Is       Search forward for text occurrences",
            b"  ^K (Ctrl+K)  Cut Text       Cut current line to clipboard buffer",
            b"  ^U (Ctrl+U)  Paste Txt      Paste clipboard buffer below cursor",
            b"  ^J (Ctrl+J)  Justify        Recalculate margins and cursor telemetry",
            b"  ^C (Ctrl+C)  Cur Pos        Report line, column, and char count",
            b"  ^R (Ctrl+R)  Read File      Reload original file from disk",
            b"  ^X (Ctrl+X)  Exit           Exit editor (prompts to save if modified)",
            b"",
            b"  Arrow Keys                  Move cursor up, down, left, right",
            b"  Enter / Backspace           Insert line breaks / delete & join lines",
            b"",
            b"  [ Press any key or ^G to close this help window ]",
            b"",
        ];

        for (idx, line) in help_lines.iter().enumerate() {
            let row = idx + 1;
            if row <= c_rows {
                let fg = if idx == 0 || idx == 15 {
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

                let len = core::cmp::min(LINE_LENS[actual_y] as usize, c_cols);
                let mut x = 0;
                let mut highlight_remaining = 0;

                while x < len {
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

                    if ch.is_ascii_digit() {
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

                    if is_operator(ch) {
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

                    if is_alpha(ch) || ch == b'_' {
                        let start = x;
                        while x < len && is_alnum(EDITOR_GRID[actual_y][x]) {
                            x += 1;
                        }
                        let word_slice = &EDITOR_GRID[actual_y][start..x];
                        let is_kw = is_keyword(word_slice);

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
                            } else if is_kw {
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
