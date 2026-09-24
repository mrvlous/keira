// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Character device forwarding for `/system/dev/tty` and `/system/dev/ptmx`.

use keira_io::vga;

/// Reads characters from active TTY line discipline or PS/2 keyboard queue.
///
/// # Safety
///
/// Accesses hardware keyboard buffer queue.
pub unsafe fn read(buf: &mut [u8]) -> Result<usize, &'static str> {
    let n = keira_io::tty::read_tty(buf);
    if n > 0 {
        Ok(n)
    } else {
        let mut read_bytes = 0;
        while read_bytes < buf.len() {
            if let Some(c) = keira_io::ps2::pop_input_char() {
                buf[read_bytes] = c;
                read_bytes += 1;
            } else {
                break;
            }
        }
        Ok(read_bytes)
    }
}

/// Writes byte slice to VGA console display.
pub fn write(buf: &[u8]) -> Result<usize, &'static str> {
    if let Ok(s) = core::str::from_utf8(buf) {
        vga::print_str(s);
    }
    Ok(buf.len())
}
