// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Status bar formatting, cursor telemetry, and line counting.

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
pub unsafe fn get_file_last_line() -> usize {
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
pub unsafe fn get_total_lines() -> usize {
    get_file_last_line() + 1
}

/// Sets status message indicating lines read from persistent storage.
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

/// Sets status bar telemetry describing cursor position, column, and total character count.
pub unsafe fn set_status_cur_pos() {
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
