// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Architecture-independent Serial Debug Port interfaces.
//!
//! Provides abstract byte-oriented character stream transmission contracts
//! implemented across 16550A UART, MMIO UART, and virtual hypervisor consoles.

/// Generic Hardware Serial Interface trait.
pub trait SerialPort {
    /// Initializes serial communication port parameters (baud rate, parity, stop bits).
    fn init(&mut self);

    /// Transmits a single byte across the serial line.
    fn write_byte(&mut self, byte: u8);

    /// Reads a single byte from the serial receiver FIFO buffer if available.
    fn read_byte(&mut self) -> Option<u8>;

    /// Transmits an entire string slice across the serial line.
    fn write_str(&mut self, s: &str) {
        for b in s.bytes() {
            self.write_byte(b);
        }
    }
}
