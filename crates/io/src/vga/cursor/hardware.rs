// SPDX-License-Identifier: GPL-2.0-only

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
    let pos = row * 80 + col;
    outb(CRTC_INDEX_PORT, 0x0F);
    outb(CRTC_DATA_PORT, (pos & 0xFF) as u8);
    outb(CRTC_INDEX_PORT, 0x0E);
    outb(CRTC_DATA_PORT, ((pos >> 8) & 0xFF) as u8);
}
