// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Pure Rust 16550A Serial UART COM1 port driver for kernel logs and host terminal debugging.

use keira_arch::cpu::{inb, outb};

const COM1: u16 = 0x3F8;

static mut SERIAL_INITIALIZED: bool = false;

/// Initialize COM1 16550A UART serial port (115200 baud, 8N1, FIFO enabled).
pub fn init() {
    unsafe {
        // 1. Disable all interrupts
        outb(COM1 + 1, 0x00);

        // 2. Enable DLAB (set baud rate divisor)
        outb(COM1 + 3, 0x80);

        // 3. Set divisor to 1 (lobyte 0x01, hibyte 0x00) -> 115200 baud
        outb(COM1 + 0, 0x01);
        outb(COM1 + 1, 0x00);

        // 4. 8 bits, no parity, 1 stop bit
        outb(COM1 + 3, 0x03);

        // 5. Enable FIFO, clear TX/RX FIFOs, 14-byte threshold
        outb(COM1 + 2, 0xC7);

        // 6. Set RTS/DSR, Auxiliary Output 2 (IRQs enabled)
        outb(COM1 + 4, 0x0B);

        SERIAL_INITIALIZED = true;
    }
}

/// Check if the serial transmitter buffer is empty and ready for a new byte.
#[inline(always)]
fn is_transmit_empty() -> bool {
    unsafe { (inb(COM1 + 5) & 0x20) != 0 }
}

/// Check if a byte has been received on the COM1 serial port (Data Ready).
#[inline(always)]
pub fn has_byte() -> bool {
    unsafe {
        if !SERIAL_INITIALIZED {
            init();
        }
        (inb(COM1 + 5) & 0x01) != 0
    }
}

/// Read a single received byte from the COM1 serial port.
#[inline(always)]
pub fn read_byte() -> u8 {
    unsafe {
        if has_byte() {
            inb(COM1)
        } else {
            0
        }
    }
}

/// Write a single ASCII byte to the COM1 serial port in pure Rust.
pub fn putchar(c: u8) {
    unsafe {
        if !SERIAL_INITIALIZED {
            init();
        }
        while !is_transmit_empty() {
            core::hint::spin_loop();
        }
        outb(COM1, c);
    }
}

/// C-compatible export for serial write.
#[no_mangle]
pub extern "C" fn serial_putchar(c: u8) {
    putchar(c);
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
