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

//! Subsystem module: matcher.

pub mod matcher {
/// Locates the start position and text slice of the last whitespace-delimited word in a buffer.
pub fn find_last_word(buf: &[u8]) -> (usize, &str) {
    let mut i = buf.len();
    while i > 0 && buf[i - 1] != b' ' {
        i -= 1;
    }
    let word_bytes = &buf[i..];
    if let Ok(s) = core::str::from_utf8(word_bytes) {
        (i, s)
    } else {
        (buf.len(), "")
    }
}

}

pub use self::matcher::*;
