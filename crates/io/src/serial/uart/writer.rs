// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Formatted string and numerical printing over serial communication channels.

use super::driver::putchar;
use core::fmt;

/// Structure implementing `core::fmt::Write` for serial UART output.
#[derive(Debug, Default, Clone, Copy)]
pub struct SerialWriter;

impl fmt::Write for SerialWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        print_str(s);
        Ok(())
    }
}

/// Writes a byte slice to the active COM1 serial port.
pub fn print(bytes: &[u8]) {
    for &b in bytes {
        putchar(b);
    }
}

/// Writes a UTF-8 string slice to the active COM1 serial port.
pub fn print_str(s: &str) {
    print(s.as_bytes());
}

/// Formats and prints a 64-bit unsigned integer in decimal radix.
pub fn print_u64(val: u64) {
    if val == 0 {
        putchar(b'0');
        return;
    }
    let mut buf = [0u8; 20];
    let mut i = 20;
    let mut temp = val;
    while temp > 0 {
        i -= 1;
        buf[i] = b'0' + (temp % 10) as u8;
        temp /= 10;
    }
    print(&buf[i..]);
}

/// Formats and prints a 64-bit unsigned integer in hexadecimal representation with `0x` prefix.
pub fn print_hex(val: u64) {
    print_str("0x");
    let chars = b"0123456789ABCDEF";
    let mut buf = [0u8; 16];
    let mut i = 16;
    let mut temp = val;
    if temp == 0 {
        putchar(b'0');
        return;
    }
    while temp > 0 {
        i -= 1;
        buf[i] = chars[(temp & 0x0F) as usize];
        temp >>= 4;
    }
    print(&buf[i..]);
}

/// Helper function to format an integer into an ASCII buffer for testing without hardware I/O.
pub fn format_u64_buffer(val: u64, buf: &mut [u8; 20]) -> usize {
    if val == 0 {
        buf[19] = b'0';
        return 19;
    }
    let mut i = 20;
    let mut temp = val;
    while temp > 0 {
        i -= 1;
        buf[i] = b'0' + (temp % 10) as u8;
        temp /= 10;
    }
    i
}

/// Helper function to format a hex value into an ASCII buffer for testing without hardware I/O.
pub fn format_hex_buffer(val: u64, buf: &mut [u8; 16]) -> usize {
    let chars = b"0123456789ABCDEF";
    if val == 0 {
        buf[15] = b'0';
        return 15;
    }
    let mut i = 16;
    let mut temp = val;
    while temp > 0 {
        i -= 1;
        buf[i] = chars[(temp & 0x0F) as usize];
        temp >>= 4;
    }
    i
}
