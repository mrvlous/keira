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

//! Subsystem module: buffer.

pub mod file {
    use keira_io::vga;

    use super::status::{get_file_last_line, set_status_msg_read};
    use crate::editor::render::metrics::{content_cols, FILE_BUF_SIZE, GRID_LINE_WIDTH};
    use crate::editor::render::view::editor_redraw;
    use crate::state::session::{
        EDITOR_CONFIRM_EXIT, EDITOR_CONFIRM_SAVE, EDITOR_FILE_BUF, EDITOR_GRID, EDITOR_HELP_MODE,
        EDITOR_MODIFIED, EDITOR_STATUS_COLOR, EDITOR_STATUS_LEN, EDITOR_STATUS_MSG, EDIT_CUR_X,
        EDIT_CUR_Y, EDIT_FILENAME, EDIT_FILENAME_LEN, EDIT_SCROLL_Y, IN_EDITOR_MODE,
        IN_SEARCH_MODE, LINE_LENS, SEARCH_BUFFER, SEARCH_LEN,
    };

    /// Save current in-memory buffer to persistent FAT16 storage.
    ///
    /// # Safety
    /// Reads static mutable editor buffers and executes VFS file operations.
    pub unsafe fn editor_save_file() -> Result<usize, &'static str> {
        let mut flat_len = 0;
        let last_y = get_file_last_line();

        for y in 0..=last_y {
            let row_len = (LINE_LENS[y] as usize).min(GRID_LINE_WIDTH);
            for &ch in EDITOR_GRID[y].iter().take(row_len) {
                if flat_len < FILE_BUF_SIZE {
                    EDITOR_FILE_BUF[flat_len] = ch;
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
    ///
    /// # Safety
    /// Initializes static mutable editor grid, file buffers and session flags.
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
}

pub use self::file::*;
pub mod status {
    use keira_io::vga;

    use crate::state::session::{
        EDITOR_STATUS_COLOR, EDITOR_STATUS_LEN, EDITOR_STATUS_MSG, EDIT_CUR_X, EDIT_CUR_Y,
        EDIT_FILENAME, EDIT_FILENAME_LEN, LINE_LENS,
    };

    /// Format an unsigned 64-bit integer into a decimal byte buffer.
    pub fn format_u64(mut val: u64, buf: &mut [u8]) -> usize {
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

    /// Append byte slice to destination buffer updating the cursor length.
    pub fn append_bytes(buf: &mut [u8], len: &mut usize, data: &[u8]) {
        let to_copy = data.len().min(buf.len().saturating_sub(*len));
        buf[*len..*len + to_copy].copy_from_slice(&data[..to_copy]);
        *len += to_copy;
    }

    /// Returns the 0-based index of the last active line in the document.
    ///
    /// # Safety
    /// Reads static mutable line length array `LINE_LENS`.
    pub unsafe fn get_file_last_line() -> usize {
        let mut last = 0;
        for (y, &len) in LINE_LENS.iter().enumerate() {
            if len > 0 {
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
    ///
    /// # Safety
    /// Inspects global editor line structures.
    pub unsafe fn get_total_lines() -> usize {
        get_file_last_line() + 1
    }

    /// Sets status message indicating lines read from persistent storage.
    ///
    /// # Safety
    /// Modifies global static editor status message buffers.
    pub unsafe fn set_status_msg_read(lines: usize) {
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

    /// Sets status message indicating lines saved to persistent storage.
    ///
    /// # Safety
    /// Modifies global static editor status message buffers.
    pub unsafe fn set_status_msg_wrote(lines: usize) {
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

    /// Sets status bar telemetry describing cursor position, column and total character count.
    ///
    /// # Safety
    /// Accesses static mutable editor cursor and text metadata.
    pub unsafe fn set_status_cur_pos() {
        let cur_line = (EDIT_CUR_Y + 1) as usize;
        let total_lines = get_total_lines();
        let mut total_chars = 0;
        let mut char_pos = 0;

        for (y, &llen_u16) in LINE_LENS.iter().enumerate().take(total_lines) {
            let llen = llen_u16 as usize;
            if y < (EDIT_CUR_Y as usize) {
                char_pos += llen + 1;
            } else if y == (EDIT_CUR_Y as usize) {
                char_pos += (EDIT_CUR_X as usize).min(llen);
            }
            total_chars += llen + 1;
        }
        total_chars = total_chars.saturating_sub(1);

        let cur_col = (EDIT_CUR_X + 1) as usize;
        let line_len = (LINE_LENS[EDIT_CUR_Y as usize] as usize + 1).max(1);
        let line_pct = cur_line.saturating_mul(100) / total_lines.max(1);
        let col_pct = cur_col.saturating_mul(100) / line_len;
        let char_pct = char_pos
            .saturating_mul(100)
            .checked_div(total_chars)
            .unwrap_or(100);

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
}

pub use self::status::*;
