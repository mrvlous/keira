// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Keira Kernel: Multi-Virtual Terminal TTY Subsystem
//!
//! Provides up to 3 independent virtual terminal instances (TTY 1, TTY 2, TTY 3)
//! with dedicated screen buffers, active session contexts, and Alt+F1/F2/F3 switching.

use crate::io::vga;

/// Active TTY index (0 = TTY1, 1 = TTY2, 2 = TTY3)
pub static mut ACTIVE_TTY: usize = 0;

/// Switch active virtual terminal (0 = TTY1, 1 = TTY2, 2 = TTY3)
pub fn switch_tty(index: usize) {
    if index > 2 {
        return;
    }
    unsafe {
        if ACTIVE_TTY == index {
            return;
        }
        ACTIVE_TTY = index;
        vga::set_color(vga::Color::LightCyan, vga::Color::Black);
        vga::print_str("[TTY] Switched to Virtual Terminal /dev/tty");
        vga::print_u64((index + 1) as u64);
        vga::print_str("\n");
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    }
}
