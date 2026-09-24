// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! POSIX termios structure and terminal attribute flags.

/// POSIX termios structure representing terminal attributes and control characters.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Termios {
    /// Input mode flags.
    pub c_iflag: u32,
    /// Output mode flags.
    pub c_oflag: u32,
    /// Control mode flags.
    pub c_cflag: u32,
    /// Local mode flags.
    pub c_lflag: u32,
    /// Line discipline.
    pub c_line: u8,
    /// Special control characters array.
    pub c_cc: [u8; 32],
    /// Input baud rate.
    pub c_ispeed: u32,
    /// Output baud rate.
    pub c_ospeed: u32,
}

impl Default for Termios {
    fn default() -> Self {
        Self {
            c_iflag: 0x0500,
            c_oflag: 0x0005,
            c_cflag: 0x00BF,
            c_lflag: 0x8A3B,
            c_line: 0,
            c_cc: [
                3, 28, 127, 21, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0,
            ],
            c_ispeed: 38400,
            c_ospeed: 38400,
        }
    }
}

/// Local mode flag: Enable signals (SIGINT, SIGQUIT, etc.).
pub const ISIG: u32 = 0o0000001;

/// Local mode flag: Canonical input processing (line-buffered).
pub const ICANON: u32 = 0o0000002;

/// Local mode flag: Echo input characters.
pub const ECHO: u32 = 0o0000010;

/// Local mode flag: Visually erase previous character on erase.
pub const ECHOE: u32 = 0o0000020;

/// Local mode flag: Echo newline after kill.
pub const ECHOK: u32 = 0o0000040;

/// Local mode flag: Echo newline even if ECHO is not set.
pub const ECHONL: u32 = 0o0000100;

/// Control character subscript index: Interrupt character (SIGINT, default Ctrl+C).
pub const VINTR: usize = 0;

/// Control character subscript index: Quit character (SIGQUIT, default Ctrl+\).
pub const VQUIT: usize = 1;

/// Control character subscript index: Erase character (Backspace, default 127).
pub const VERASE: usize = 2;

/// Control character subscript index: Kill character (default Ctrl+U).
pub const VKILL: usize = 3;

/// Control character subscript index: End-of-file character (default Ctrl+D).
pub const VEOF: usize = 4;
