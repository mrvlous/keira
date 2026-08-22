// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Architecture-independent Serial Debug Port traits.

/// Generic Hardware Serial Interface trait.
pub trait SerialPort {
    /// Initialize serial communication port.
    fn init(&mut self);

    /// Transmit a single byte across the serial line.
    fn write_byte(&mut self, byte: u8);

    /// Read a single byte from the serial receiver buffer.
    fn read_byte(&mut self) -> Option<u8>;

    /// Transmit a string slice across the serial line.
    fn write_str(&mut self, s: &str) {
        for b in s.bytes() {
            self.write_byte(b);
        }
    }
}
