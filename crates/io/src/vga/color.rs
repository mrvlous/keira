// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! VGA 16-color palette and RGB color mapping.

/// Standard 16-color VGA palette enumeration.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGrey = 7,
    DarkGrey = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    LightMagenta = 13,
    Yellow = 14,
    White = 15,
}

impl Color {
    /// Convert VGA 16-color palette index to 32-bpp RGB color integer.
    pub const fn to_rgb(self) -> u32 {
        match self {
            Color::Black => 0x000000,
            Color::Blue => 0x0000AA,
            Color::Green => 0x00AA00,
            Color::Cyan => 0x00AAAA,
            Color::Red => 0xAA0000,
            Color::Magenta => 0xAA00AA,
            Color::Brown => 0xAA5500,
            Color::LightGrey => 0xAAAAAA,
            Color::DarkGrey => 0x555555,
            Color::LightBlue => 0x5555FF,
            Color::LightGreen => 0x55FF55,
            Color::LightCyan => 0x55FFFF,
            Color::LightRed => 0xFF5555,
            Color::LightMagenta => 0xFF55FF,
            Color::Yellow => 0xFFFF55,
            Color::White => 0xFFFFFF,
        }
    }
}
