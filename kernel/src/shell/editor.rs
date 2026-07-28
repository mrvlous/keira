// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Keira Kernel: VGA Code Editor

use super::state::*;
use super::{KEY_DOWN, KEY_LEFT, KEY_RIGHT, KEY_UP};
use crate::io::vga;

unsafe fn editor_save_file() -> Result<(), &'static str> {
    let mut flat_buf = [0u8; 2048];
    let mut flat_len = 0;

    let mut last_y = 0;
    for y in 0..23 {
        if LINE_LENS[y] > 0 {
            last_y = y;
        }
    }

    for y in 0..=last_y {
        let row_len = LINE_LENS[y] as usize;
        for x in 0..row_len {
            if flat_len < 2048 {
                flat_buf[flat_len] = EDITOR_GRID[y][x];
                flat_len += 1;
            }
        }

        if y < last_y {
            if flat_len < 2048 {
                flat_buf[flat_len] = b'\n';
                flat_len += 1;
            }
        }
    }

    while flat_len > 0 && flat_buf[flat_len - 1] == b'\n' {
        flat_len -= 1;
    }

    let filename_slice = &EDIT_FILENAME[..EDIT_FILENAME_LEN];
    let filename_str =
        core::str::from_utf8(filename_slice).map_err(|_| "Invalid filename encoding")?;

    crate::fs::vfs::write_file(filename_str, &flat_buf[..flat_len])?;
    Ok(())
}

pub unsafe fn editor_start(filename: &str) -> Result<(), &'static str> {
    EDIT_FILENAME = [0; 12];
    EDIT_FILENAME_LEN = core::cmp::min(filename.len(), 12);
    EDIT_FILENAME[..EDIT_FILENAME_LEN].copy_from_slice(filename.as_bytes());

    EDITOR_GRID = [[b' '; 80]; 23];
    LINE_LENS = [0; 23];
    EDIT_CUR_X = 0;
    EDIT_CUR_Y = 0;
    EDITOR_CONFIRM_EXIT = false;
    EDITOR_CONFIRM_SAVE = false;
    IN_SEARCH_MODE = false;
    SEARCH_LEN = 0;
    SEARCH_BUFFER = [0; 16];

    let mut file_buf = [0u8; 2048];
    match crate::fs::vfs::read_file(filename, &mut file_buf) {
        Ok(bytes_read) => {
            let mut x = 0;
            let mut y = 0;
            for &b in &file_buf[..bytes_read] {
                if b == b'\n' {
                    if y < 23 {
                        LINE_LENS[y] = x as u16;
                    }
                    x = 0;
                    y += 1;
                    if y >= 23 {
                        break;
                    }
                } else if b == b'\r' {
                    // skip CR
                } else {
                    if x < 75 {
                        EDITOR_GRID[y][x] = b;
                        x += 1;
                    }
                }
            }
            if y < 23 {
                LINE_LENS[y] = x as u16;
            }
        }
        Err(_) => {
            if let Err(e) = crate::fs::fat::create_file(filename) {
                if e != "File or directory already exists" {
                    return Err(e);
                }
            }
        }
    }

    IN_EDITOR_MODE = true;
    editor_redraw();
    Ok(())
}

pub unsafe fn editor_redraw() {
    vga::init();

    // 1. Draw top bar (Header)
    vga::set_color(vga::Color::White, vga::Color::DarkGrey);
    vga::print_str("  Keira Text Editor ");
    vga::print_str(env!("CARGO_PKG_VERSION"));
    vga::print_str("  |  File: ");
    let filename_slice = &EDIT_FILENAME[..EDIT_FILENAME_LEN];
    if let Ok(name_str) = core::str::from_utf8(filename_slice) {
        vga::print_str(name_str);
    }
    vga::print_str(" ");

    let mut current_col = vga::get_cursor_col();
    while current_col < 80 {
        vga::print_str(" ");
        current_col += 1;
    }

    // 2. Draw grid content with syntax highlighting and line numbers
    for y in 0..23 {
        vga::set_cursor_pos((y + 1) as u16, 0);

        // Render line number gutter (e.g. " 1 | ")
        let val = (y + 1) as u8;
        let ten = val / 10;
        let one = val % 10;
        let ten_char = if ten == 0 { b' ' } else { b'0' + ten };
        let one_char = b'0' + one;
        let gutter = [ten_char, one_char, b' ', b'|', b' '];
        if let Ok(gutter_str) = core::str::from_utf8(&gutter) {
            vga::set_color(vga::Color::DarkGrey, vga::Color::Black);
            vga::print_str(gutter_str);
        }

        let len = core::cmp::min(LINE_LENS[y] as usize, 75);
        let mut x = 0;
        let mut highlight_remaining = 0;

        while x < len {
            // Check if search match starts at this position
            if SEARCH_LEN > 0 && highlight_remaining == 0 && x + SEARCH_LEN <= len {
                let mut matched = true;
                for i in 0..SEARCH_LEN {
                    if EDITOR_GRID[y][x + i] != SEARCH_BUFFER[i] {
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
            let c = EDITOR_GRID[y][x];

            // 1. Highlight numbers
            if c >= b'0' && c <= b'9' {
                vga::set_color(
                    if fg_override {
                        vga::Color::Black
                    } else {
                        vga::Color::LightRed
                    },
                    bg_color,
                );
                let s = [c];
                if let Ok(c_str) = core::str::from_utf8(&s) {
                    vga::print_str(c_str);
                }
                x += 1;
                if highlight_remaining > 0 {
                    highlight_remaining -= 1;
                }
                continue;
            }

            // 2. Highlight strings
            if c == b'"' {
                vga::set_color(
                    if fg_override {
                        vga::Color::Black
                    } else {
                        vga::Color::Yellow
                    },
                    bg_color,
                );
                let s = [c];
                if let Ok(c_str) = core::str::from_utf8(&s) {
                    vga::print_str(c_str);
                }
                x += 1;
                if highlight_remaining > 0 {
                    highlight_remaining -= 1;
                }
                while x < len {
                    if SEARCH_LEN > 0 && highlight_remaining == 0 && x + SEARCH_LEN <= len {
                        let mut matched = true;
                        for i in 0..SEARCH_LEN {
                            if EDITOR_GRID[y][x + i] != SEARCH_BUFFER[i] {
                                matched = false;
                                break;
                            }
                        }
                        if matched {
                            highlight_remaining = SEARCH_LEN;
                        }
                    }
                    let str_bg = if highlight_remaining > 0 {
                        vga::Color::Yellow
                    } else {
                        vga::Color::Black
                    };
                    let str_fg = if highlight_remaining > 0 {
                        vga::Color::Black
                    } else {
                        vga::Color::Yellow
                    };
                    vga::set_color(str_fg, str_bg);

                    let sc = EDITOR_GRID[y][x];
                    let s_sc = [sc];
                    if let Ok(c_str) = core::str::from_utf8(&s_sc) {
                        vga::print_str(c_str);
                    }
                    x += 1;
                    if highlight_remaining > 0 {
                        highlight_remaining -= 1;
                    }
                    if sc == b'"' {
                        break;
                    }
                }
                continue;
            }
            if c == b'\'' {
                vga::set_color(
                    if fg_override {
                        vga::Color::Black
                    } else {
                        vga::Color::Yellow
                    },
                    bg_color,
                );
                let s = [c];
                if let Ok(c_str) = core::str::from_utf8(&s) {
                    vga::print_str(c_str);
                }
                x += 1;
                if highlight_remaining > 0 {
                    highlight_remaining -= 1;
                }
                while x < len {
                    if SEARCH_LEN > 0 && highlight_remaining == 0 && x + SEARCH_LEN <= len {
                        let mut matched = true;
                        for i in 0..SEARCH_LEN {
                            if EDITOR_GRID[y][x + i] != SEARCH_BUFFER[i] {
                                matched = false;
                                break;
                            }
                        }
                        if matched {
                            highlight_remaining = SEARCH_LEN;
                        }
                    }
                    let str_bg = if highlight_remaining > 0 {
                        vga::Color::Yellow
                    } else {
                        vga::Color::Black
                    };
                    let str_fg = if highlight_remaining > 0 {
                        vga::Color::Black
                    } else {
                        vga::Color::Yellow
                    };
                    vga::set_color(str_fg, str_bg);

                    let sc = EDITOR_GRID[y][x];
                    let s_sc = [sc];
                    if let Ok(c_str) = core::str::from_utf8(&s_sc) {
                        vga::print_str(c_str);
                    }
                    x += 1;
                    if highlight_remaining > 0 {
                        highlight_remaining -= 1;
                    }
                    if sc == b'\'' {
                        break;
                    }
                }
                continue;
            }

            // 3. Highlight comments
            if c == b'/' && x + 1 < len && EDITOR_GRID[y][x + 1] == b'/' {
                vga::set_color(
                    if fg_override {
                        vga::Color::Black
                    } else {
                        vga::Color::LightGreen
                    },
                    bg_color,
                );
                while x < len {
                    if SEARCH_LEN > 0 && highlight_remaining == 0 && x + SEARCH_LEN <= len {
                        let mut matched = true;
                        for i in 0..SEARCH_LEN {
                            if EDITOR_GRID[y][x + i] != SEARCH_BUFFER[i] {
                                matched = false;
                                break;
                            }
                        }
                        if matched {
                            highlight_remaining = SEARCH_LEN;
                        }
                    }
                    let cmt_bg = if highlight_remaining > 0 {
                        vga::Color::Yellow
                    } else {
                        vga::Color::Black
                    };
                    let cmt_fg = if highlight_remaining > 0 {
                        vga::Color::Black
                    } else {
                        vga::Color::LightGreen
                    };
                    vga::set_color(cmt_fg, cmt_bg);

                    let sc = EDITOR_GRID[y][x];
                    let s_sc = [sc];
                    if let Ok(c_str) = core::str::from_utf8(&s_sc) {
                        vga::print_str(c_str);
                    }
                    x += 1;
                    if highlight_remaining > 0 {
                        highlight_remaining -= 1;
                    }
                }
                continue;
            }

            // 4. Highlight symbols / operators
            if c == b'='
                || c == b'+'
                || c == b'-'
                || c == b'*'
                || c == b'/'
                || c == b'%'
                || c == b'&'
                || c == b'|'
                || c == b'^'
                || c == b'!'
                || c == b'<'
                || c == b'>'
            {
                vga::set_color(
                    if fg_override {
                        vga::Color::Black
                    } else {
                        vga::Color::LightMagenta
                    },
                    bg_color,
                );
                let s = [c];
                if let Ok(c_str) = core::str::from_utf8(&s) {
                    vga::print_str(c_str);
                }
                x += 1;
                if highlight_remaining > 0 {
                    highlight_remaining -= 1;
                }
                continue;
            }

            // 5. Highlight words (keywords vs identifier)
            let is_alpha = |b: u8| -> bool { (b >= b'a' && b <= b'z') || (b >= b'A' && b <= b'Z') };
            let is_alnum = |b: u8| -> bool { is_alpha(b) || (b >= b'0' && b <= b'9') || b == b'_' };

            if is_alpha(c) || c == b'_' {
                let start = x;
                while x < len && is_alnum(EDITOR_GRID[y][x]) {
                    x += 1;
                }
                let word_slice = &EDITOR_GRID[y][start..x];
                let is_keyword = match word_slice {
                    b"fn" | b"let" | b"struct" | b"impl" | b"pub" | b"for" | b"if" | b"else"
                    | b"match" | b"return" | b"loop" | b"mut" | b"static" | b"const" | b"use"
                    | b"mod" | b"as" | b"enum" | b"type" | b"true" | b"false" => true,
                    _ => false,
                };

                for (offset, &b) in word_slice.iter().enumerate() {
                    let word_char_x = start + offset;
                    if SEARCH_LEN > 0 && highlight_remaining == 0 && word_char_x + SEARCH_LEN <= len
                    {
                        let mut matched = true;
                        for i in 0..SEARCH_LEN {
                            if EDITOR_GRID[y][word_char_x + i] != SEARCH_BUFFER[i] {
                                matched = false;
                                break;
                            }
                        }
                        if matched {
                            highlight_remaining = SEARCH_LEN;
                        }
                    }
                    let w_bg = if highlight_remaining > 0 {
                        vga::Color::Yellow
                    } else {
                        vga::Color::Black
                    };
                    let w_fg = if highlight_remaining > 0 {
                        vga::Color::Black
                    } else if is_keyword {
                        vga::Color::LightBlue
                    } else {
                        vga::Color::White
                    };
                    vga::set_color(w_fg, w_bg);

                    let s_b = [b];
                    if let Ok(c_str) = core::str::from_utf8(&s_b) {
                        vga::print_str(c_str);
                    }
                    if highlight_remaining > 0 {
                        highlight_remaining -= 1;
                    }
                }
                continue;
            }

            // Default character
            vga::set_color(
                if fg_override {
                    vga::Color::Black
                } else {
                    vga::Color::White
                },
                bg_color,
            );
            let s = [c];
            if let Ok(c_str) = core::str::from_utf8(&s) {
                vga::print_str(c_str);
            }
            x += 1;
            if highlight_remaining > 0 {
                highlight_remaining -= 1;
            }
        }

        // Pad rest of line with space
        vga::set_color(vga::Color::White, vga::Color::Black);
        let mut pad = len;
        while pad < 75 {
            vga::print_str(" ");
            pad += 1;
        }
    }

    // 3. Draw bottom bar (Help/Status/Search)
    vga::set_cursor_pos(24, 0);
    if EDITOR_CONFIRM_SAVE {
        vga::set_color(vga::Color::White, vga::Color::DarkGrey);
        vga::print_str("  Save changes? [Y] Yes  [N] No  [C] Cancel");
    } else if IN_SEARCH_MODE {
        vga::set_color(vga::Color::White, vga::Color::DarkGrey);
        vga::print_str("  Search: ");
        let search_slice = &SEARCH_BUFFER[..SEARCH_LEN];
        if let Ok(s) = core::str::from_utf8(search_slice) {
            vga::print_str(s);
        }
    } else if EDITOR_STATUS_LEN > 0 {
        vga::set_color(EDITOR_STATUS_COLOR, vga::Color::DarkGrey);
        vga::print_str("  ");
        let status_slice = &EDITOR_STATUS_MSG[..EDITOR_STATUS_LEN];
        if let Ok(s) = core::str::from_utf8(status_slice) {
            vga::print_str(s);
        }
    } else {
        vga::set_color(vga::Color::White, vga::Color::DarkGrey);
        vga::print_str(
            "  ESC: Exit  |  Ctrl+F: Search  |  F3/Ctrl+S: Save  |  F10/Ctrl+Q: Save & Exit",
        );
    }

    let mut current_col = vga::get_cursor_col();
    while current_col < 80 {
        vga::print_str(" ");
        current_col += 1;
    }

    vga::set_color(vga::Color::LightGrey, vga::Color::Black);

    if EDITOR_CONFIRM_SAVE {
        vga::set_cursor_pos(24, 45);
    } else if IN_SEARCH_MODE {
        vga::set_cursor_pos(24, (10 + SEARCH_LEN) as u16);
    } else {
        vga::set_cursor_pos(EDIT_CUR_Y + 1, EDIT_CUR_X + 5);
    }
}

pub unsafe fn editor_handle_keypress(c: u8) {
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
                super::print_prompt();
            }
            b'n' | b'N' => {
                IN_EDITOR_MODE = false;
                vga::init();
                super::print_prompt();
            }
            b'c' | b'C' | 27 => {
                EDITOR_CONFIRM_SAVE = false;
                editor_redraw();
            }
            _ => {}
        }
        return;
    }

    if EDITOR_CONFIRM_EXIT {
        IN_EDITOR_MODE = false;
        vga::init();
        super::print_prompt();
        return;
    }

    if IN_SEARCH_MODE {
        match c {
            27 => {
                // Esc: Exit search mode
                IN_SEARCH_MODE = false;
                SEARCH_LEN = 0;
                SEARCH_BUFFER = [0; 16];
                editor_redraw();
            }
            10 | 13 => {
                // Enter: Execute search
                let mut found = false;
                let term = &SEARCH_BUFFER[..SEARCH_LEN];
                if SEARCH_LEN > 0 {
                    'outer: for y in 0..23 {
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
                                    found = true;
                                    break 'outer;
                                }
                            }
                        }
                    }
                }
                if !found && SEARCH_LEN > 0 {
                    let msg = b"Search term not found!";
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
                }
                editor_redraw();
            }
            32..=126 => {
                // Printable characters
                if SEARCH_LEN < 16 {
                    SEARCH_BUFFER[SEARCH_LEN] = c;
                    SEARCH_LEN += 1;
                }
                editor_redraw();
            }
            _ => {}
        }
        return;
    }

    // Ctrl+F (6) Search shortcut
    if c == 6 {
        IN_SEARCH_MODE = true;
        SEARCH_LEN = 0;
        SEARCH_BUFFER = [0; 16];
        EDITOR_STATUS_LEN = 0;
        editor_redraw();
        return;
    }

    // Ctrl+S (19) or F3 (0x84) Quick Save shortcut
    if c == 19 || c == 0x84 {
        match editor_save_file() {
            Ok(_) => {
                let msg = b"File saved successfully!";
                EDITOR_STATUS_LEN = msg.len();
                EDITOR_STATUS_MSG[..msg.len()].copy_from_slice(msg);
                EDITOR_STATUS_COLOR = vga::Color::LightGreen;
            }
            Err(e) => {
                let mut msg = [0u8; 40];
                let prefix = b"Error: ";
                let mut len = prefix.len();
                msg[..len].copy_from_slice(prefix);
                let e_bytes = e.as_bytes();
                let to_copy = core::cmp::min(e_bytes.len(), 40 - len);
                msg[len..len + to_copy].copy_from_slice(&e_bytes[..to_copy]);
                len += to_copy;
                EDITOR_STATUS_LEN = len;
                EDITOR_STATUS_MSG[..len].copy_from_slice(&msg[..len]);
                EDITOR_STATUS_COLOR = vga::Color::LightRed;
            }
        }
        editor_redraw();
        return;
    }

    // Ctrl+Q (17) or F10 (0x85) Save & Exit shortcut
    if c == 17 || c == 0x85 {
        if let Err(e) = editor_save_file() {
            vga::init();
            vga::set_color(vga::Color::LightRed, vga::Color::Black);
            vga::print_str("Error saving file: ");
            vga::print_str(e);
            vga::print_str("\nPress any key to return...\n");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            EDITOR_CONFIRM_SAVE = false;
            EDITOR_CONFIRM_EXIT = true;
        } else {
            IN_EDITOR_MODE = false;
            vga::init();
            super::print_prompt();
        }
        return;
    }

    EDITOR_STATUS_LEN = 0;

    match c {
        27 => {
            EDITOR_CONFIRM_SAVE = true;
            editor_redraw();
        }
        9 => {
            // Tab key: Insert 4 spaces
            let y = EDIT_CUR_Y as usize;
            let mut x = EDIT_CUR_X as usize;
            let mut len = LINE_LENS[y] as usize;
            for _ in 0..4 {
                if len < 75 {
                    for i in (x..len).rev() {
                        EDITOR_GRID[y][i + 1] = EDITOR_GRID[y][i];
                    }
                    EDITOR_GRID[y][x] = b' ';
                    LINE_LENS[y] += 1;
                    len += 1;
                    if EDIT_CUR_X < 74 {
                        EDIT_CUR_X += 1;
                        x += 1;
                    }
                }
            }
            editor_redraw();
        }
        10 | 13 => {
            if EDIT_CUR_Y < 22 {
                let cur_y = EDIT_CUR_Y as usize;
                let cur_x = EDIT_CUR_X as usize;
                let cur_len = LINE_LENS[cur_y] as usize;

                for r in (cur_y + 1..22).rev() {
                    EDITOR_GRID[r + 1] = EDITOR_GRID[r];
                    LINE_LENS[r + 1] = LINE_LENS[r];
                }

                let next_y = cur_y + 1;
                EDITOR_GRID[next_y] = [b' '; 80];
                LINE_LENS[next_y] = 0;

                if cur_x < cur_len {
                    let to_move = cur_len - cur_x;
                    for i in 0..to_move {
                        EDITOR_GRID[next_y][i] = EDITOR_GRID[cur_y][cur_x + i];
                        EDITOR_GRID[cur_y][cur_x + i] = b' ';
                    }
                    LINE_LENS[next_y] = to_move as u16;
                }
                LINE_LENS[cur_y] = cur_x as u16;

                EDIT_CUR_Y += 1;
                EDIT_CUR_X = 0;
                editor_redraw();
            }
        }
        8 => {
            let y = EDIT_CUR_Y as usize;
            if EDIT_CUR_X > 0 {
                let x = EDIT_CUR_X as usize;
                let len = LINE_LENS[y] as usize;
                for i in x..len {
                    EDITOR_GRID[y][i - 1] = EDITOR_GRID[y][i];
                }
                if len > 0 {
                    EDITOR_GRID[y][len - 1] = b' ';
                    LINE_LENS[y] -= 1;
                }
                EDIT_CUR_X -= 1;
                editor_redraw();
            } else if EDIT_CUR_Y > 0 {
                let prev_y = (EDIT_CUR_Y - 1) as usize;
                let prev_len = LINE_LENS[prev_y] as usize;
                let cur_len = LINE_LENS[y] as usize;

                let space_left = 75 - prev_len;
                let to_copy = core::cmp::min(cur_len, space_left);

                for i in 0..to_copy {
                    EDITOR_GRID[prev_y][prev_len + i] = EDITOR_GRID[y][i];
                }
                LINE_LENS[prev_y] += to_copy as u16;

                if cur_len <= space_left {
                    for r in y..22 {
                        EDITOR_GRID[r] = EDITOR_GRID[r + 1];
                        LINE_LENS[r] = LINE_LENS[r + 1];
                    }
                    EDITOR_GRID[22] = [b' '; 80];
                    LINE_LENS[22] = 0;

                    EDIT_CUR_Y -= 1;
                    EDIT_CUR_X = prev_len as u16;
                } else {
                    let shifted = to_copy;
                    for i in 0..(cur_len - shifted) {
                        EDITOR_GRID[y][i] = EDITOR_GRID[y][i + shifted];
                    }
                    for i in (cur_len - shifted)..80 {
                        EDITOR_GRID[y][i] = b' ';
                    }
                    LINE_LENS[y] -= shifted as u16;

                    EDIT_CUR_Y -= 1;
                    EDIT_CUR_X = prev_len as u16;
                }
                editor_redraw();
            }
        }
        KEY_UP => {
            if EDIT_CUR_Y > 0 {
                EDIT_CUR_Y -= 1;
                let len = LINE_LENS[EDIT_CUR_Y as usize];
                if EDIT_CUR_X > len {
                    EDIT_CUR_X = len;
                }
                editor_redraw();
            }
        }
        KEY_DOWN => {
            if EDIT_CUR_Y < 22 {
                EDIT_CUR_Y += 1;
                let len = LINE_LENS[EDIT_CUR_Y as usize];
                if EDIT_CUR_X > len {
                    EDIT_CUR_X = len;
                }
                editor_redraw();
            }
        }
        KEY_LEFT => {
            // Arrow Left
            if EDIT_CUR_X > 0 {
                EDIT_CUR_X -= 1;
                editor_redraw();
            } else if EDIT_CUR_Y > 0 {
                EDIT_CUR_Y -= 1;
                EDIT_CUR_X = LINE_LENS[EDIT_CUR_Y as usize];
                editor_redraw();
            }
        }
        KEY_RIGHT => {
            // Arrow Right
            let len = LINE_LENS[EDIT_CUR_Y as usize];
            if EDIT_CUR_X < len {
                EDIT_CUR_X += 1;
                editor_redraw();
            } else if EDIT_CUR_Y < 22 {
                EDIT_CUR_Y += 1;
                EDIT_CUR_X = 0;
                editor_redraw();
            }
        }
        32..=126 => {
            let y = EDIT_CUR_Y as usize;
            let x = EDIT_CUR_X as usize;
            let len = LINE_LENS[y] as usize;

            if len < 75 {
                for i in (x..len).rev() {
                    EDITOR_GRID[y][i + 1] = EDITOR_GRID[y][i];
                }
                EDITOR_GRID[y][x] = c;
                LINE_LENS[y] += 1;

                if EDIT_CUR_X < 74 {
                    EDIT_CUR_X += 1;
                } else if EDIT_CUR_Y < 22 {
                    EDIT_CUR_X = 0;
                    EDIT_CUR_Y += 1;
                }
                editor_redraw();
            }
        }
        _ => {}
    }
}
