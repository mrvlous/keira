// SPDX-License-Identifier: GPL-2.0-only

//! Word extraction and prefix matching routines for autocomplete.

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
