// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! 16550A Serial UART low-level port I/O hardware driver.

use core::sync::atomic::{AtomicBool, Ordering};
use keira_arch::cpu::{inb, outb};

/// Base port address for COM1 standard 16550A serial UART.
pub const COM1: u16 = 0x3F8;

static SERIAL_INITIALIZED: AtomicBool = AtomicBool::new(false);

/// Initializes the COM1 16550A UART serial port at 115200 baud, 8 data bits, no parity, 1 stop bit, with FIFO enabled.
pub fn init() {
    unsafe {
        // 1. Disable all interrupts
        outb(COM1 + 1, 0x00);

        // 2. Enable DLAB (set baud rate divisor latch)
        outb(COM1 + 3, 0x80);

        // 3. Set divisor to 1 (low byte 0x01, high byte 0x00) -> 115200 baud
        outb(COM1 + 0, 0x01);
        outb(COM1 + 1, 0x00);

        // 4. 8 bits, no parity, 1 stop bit (8N1)
        outb(COM1 + 3, 0x03);

        // 5. Enable FIFO, clear TX/RX FIFOs, set 14-byte threshold
        outb(COM1 + 2, 0xC7);

        // 6. Set RTS/DSR, Auxiliary Output 2 (IRQs enabled)
        outb(COM1 + 4, 0x0B);

        SERIAL_INITIALIZED.store(true, Ordering::SeqCst);
    }
}

/// Checks whether the serial transmitter holding register is empty and ready to accept a new byte.
#[inline(always)]
pub fn is_transmit_empty() -> bool {
    #[cfg(not(target_os = "none"))]
    {
        true
    }
    #[cfg(target_os = "none")]
    unsafe {
        (inb(COM1 + 5) & 0x20) != 0
    }
}

/// Checks whether an unread byte is waiting in the receiver FIFO (Data Ready status bit).
#[inline(always)]
pub fn has_byte() -> bool {
    if !SERIAL_INITIALIZED.load(Ordering::Relaxed) {
        init();
    }
    // Port I/O requires unsafe block, caller guarantees valid port access.
    unsafe { (inb(COM1 + 5) & 0x01) != 0 }
}

/// Reads a single received byte from the COM1 receiver buffer. Returns 0 if no byte is present.
#[inline(always)]
pub fn read_byte() -> u8 {
    if has_byte() {
        unsafe { inb(COM1) }
    } else {
        0
    }
}

/// Transmits a single byte through the COM1 serial transmitter.
pub fn putchar(c: u8) {
    if !SERIAL_INITIALIZED.load(Ordering::Relaxed) {
        init();
    }
    while !is_transmit_empty() {
        core::hint::spin_loop();
    }
    unsafe {
        outb(COM1, c);
    }
}

/// Foreign function export for transmitting a single ASCII byte over serial.
#[no_mangle]
pub extern "C" fn serial_putchar(c: u8) {
    putchar(c);
}
