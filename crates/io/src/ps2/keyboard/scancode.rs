// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! PS/2 Keyboard Scancode Set 1 tables and layout definitions.

/// Scancode for Left Shift key make code.
pub const KEY_LSHIFT: u8 = 0x2A;

/// Scancode for Right Shift key make code.
pub const KEY_RSHIFT: u8 = 0x36;

/// Scancode for Left Control key make code.
pub const KEY_LCTRL: u8 = 0x1D;

/// Scancode for Up Arrow navigation key.
pub const KEY_UP: u8 = 0x48;

/// Scancode for Down Arrow navigation key.
pub const KEY_DOWN: u8 = 0x50;

/// Scancode for Left Arrow navigation key.
pub const KEY_LEFT: u8 = 0x4B;

/// Scancode for Right Arrow navigation key.
pub const KEY_RIGHT: u8 = 0x4D;

/// Scancode for Function Key F3.
pub const KEY_F3: u8 = 0x3D;

/// Scancode for Function Key F10.
pub const KEY_F10: u8 = 0x44;

/// Standard US QWERTY unshifted ASCII scancode translation table.
pub const KBD_US_LAYOUT: [u8; 128] = {
    let mut table = [0u8; 128];
    table[0x01] = 27;
    table[0x02] = b'1';
    table[0x03] = b'2';
    table[0x04] = b'3';
    table[0x05] = b'4';
    table[0x06] = b'5';
    table[0x07] = b'6';
    table[0x08] = b'7';
    table[0x09] = b'8';
    table[0x0A] = b'9';
    table[0x0B] = b'0';
    table[0x0C] = b'-';
    table[0x0D] = b'=';
    table[0x0E] = b'\x08';
    table[0x0F] = b'\t';
    table[0x10] = b'q';
    table[0x11] = b'w';
    table[0x12] = b'e';
    table[0x13] = b'r';
    table[0x14] = b't';
    table[0x15] = b'y';
    table[0x16] = b'u';
    table[0x17] = b'i';
    table[0x18] = b'o';
    table[0x19] = b'p';
    table[0x1A] = b'[';
    table[0x1B] = b']';
    table[0x1C] = b'\n';
    table[0x1E] = b'a';
    table[0x1F] = b's';
    table[0x20] = b'd';
    table[0x21] = b'f';
    table[0x22] = b'g';
    table[0x23] = b'h';
    table[0x24] = b'j';
    table[0x25] = b'k';
    table[0x26] = b'l';
    table[0x27] = b';';
    table[0x28] = b'\'';
    table[0x29] = b'`';
    table[0x2B] = b'\\';
    table[0x2C] = b'z';
    table[0x2D] = b'x';
    table[0x2E] = b'c';
    table[0x2F] = b'v';
    table[0x30] = b'b';
    table[0x31] = b'n';
    table[0x32] = b'm';
    table[0x33] = b',';
    table[0x34] = b'.';
    table[0x35] = b'/';
    table[0x37] = b'*';
    table[0x39] = b' ';
    table
};

/// Standard US QWERTY shifted ASCII scancode translation table.
pub const KBD_US_SHIFTED_LAYOUT: [u8; 128] = {
    let mut table = [0u8; 128];
    table[0x01] = 27;
    table[0x02] = b'!';
    table[0x03] = b'@';
    table[0x04] = b'#';
    table[0x05] = b'$';
    table[0x06] = b'%';
    table[0x07] = b'^';
    table[0x08] = b'&';
    table[0x09] = b'*';
    table[0x0A] = b'(';
    table[0x0B] = b')';
    table[0x0C] = b'_';
    table[0x0D] = b'+';
    table[0x0E] = b'\x08';
    table[0x0F] = b'\t';
    table[0x10] = b'Q';
    table[0x11] = b'W';
    table[0x12] = b'E';
    table[0x13] = b'R';
    table[0x14] = b'T';
    table[0x15] = b'Y';
    table[0x16] = b'U';
    table[0x17] = b'I';
    table[0x18] = b'O';
    table[0x19] = b'P';
    table[0x1A] = b'{';
    table[0x1B] = b'}';
    table[0x1C] = b'\n';
    table[0x1E] = b'A';
    table[0x1F] = b'S';
    table[0x20] = b'D';
    table[0x21] = b'F';
    table[0x22] = b'G';
    table[0x23] = b'H';
    table[0x24] = b'J';
    table[0x25] = b'K';
    table[0x26] = b'L';
    table[0x27] = b':';
    table[0x28] = b'"';
    table[0x29] = b'~';
    table[0x2B] = b'|';
    table[0x2C] = b'Z';
    table[0x2D] = b'X';
    table[0x2E] = b'C';
    table[0x2F] = b'V';
    table[0x30] = b'B';
    table[0x31] = b'N';
    table[0x32] = b'M';
    table[0x33] = b'<';
    table[0x34] = b'>';
    table[0x35] = b'?';
    table[0x37] = b'*';
    table[0x39] = b' ';
    table
};
