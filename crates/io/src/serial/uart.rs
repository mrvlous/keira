// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! 16550A Serial UART COM1 port driver for kernel logs and host terminal debugging.

extern "C" {
    fn serial_putchar(c: core::ffi::c_char);
}

/// Write a single ASCII byte to the COM1 serial port.
pub fn putchar(c: u8) {
    unsafe {
        serial_putchar(c as core::ffi::c_char);
    }
}

/// Write a byte string to the COM1 serial port.
pub fn print(s: &[u8]) {
    for &byte in s {
        putchar(byte);
    }
}

/// Write a string slice to the COM1 serial port.
pub fn print_str(s: &str) {
    print(s.as_bytes());
}

/// Print a 64-bit unsigned integer to the serial port.
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

/// Print a 64-bit hex value to the serial port.
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
