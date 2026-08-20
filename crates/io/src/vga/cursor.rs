// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Cursor tracking, blinking state, and graphical mouse cursor bitmaps.

pub const MOUSE_CURSOR_BODY: [u16; 16] = [
    0b0000000000000000,
    0b0100000000000000,
    0b0110000000000000,
    0b0111000000000000,
    0b0111100000000000,
    0b0111110000000000,
    0b0111111000000000,
    0b0111111100000000,
    0b0111111110000000,
    0b0111111111000000,
    0b0111110000000000,
    0b0110110000000000,
    0b0000110000000000,
    0b0000011000000000,
    0b0000011000000000,
    0b0000000000000000,
];

pub const MOUSE_CURSOR_OUTLINE: [u16; 16] = [
    0b1000000000000000,
    0b1010000000000000,
    0b1001000000000000,
    0b1000100000000000,
    0b1000010000000000,
    0b1000001000000000,
    0b1000000100000000,
    0b1000000010000000,
    0b1000000001000000,
    0b1000000000100000,
    0b1000001111000000,
    0b1001001000000000,
    0b0110100100000000,
    0b0000100100000000,
    0b0000011000000000,
    0b0000011000000000,
];

pub static mut CURSOR_X: u32 = 0;
pub static mut CURSOR_Y: u32 = 0;
pub static mut CURSOR_BLINK_STATE: bool = true;

pub static mut MOUSE_X: u32 = 9999;
pub static mut MOUSE_Y: u32 = 9999;
pub static mut MOUSE_VISIBLE: bool = false;
pub static mut SAVED_MOUSE_PIXELS: [u32; 192] = [0; 192];
