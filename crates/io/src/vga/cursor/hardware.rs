// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Hardware CRT Controller (CRTC) text-mode cursor manipulation.

use keira_arch::cpu::outb;

/// I/O port address for VGA CRTC index register.
pub const CRTC_INDEX_PORT: u16 = 0x3D4;

/// I/O port address for VGA CRTC data register.
pub const CRTC_DATA_PORT: u16 = 0x3D5;

/// Updates hardware text-mode blinking cursor position via CRTC index registers 0x0E and 0x0F.
///
/// # Safety
///
/// Performs direct I/O port writes to the VGA CRT controller.
pub unsafe fn vga_set_hardware_cursor(col: u16, row: u16) {
    let pos = (row * 80 + col) as u16;
    outb(CRTC_INDEX_PORT, 0x0F);
    outb(CRTC_DATA_PORT, (pos & 0xFF) as u8);
    outb(CRTC_INDEX_PORT, 0x0E);
    outb(CRTC_DATA_PORT, ((pos >> 8) & 0xFF) as u8);
}
