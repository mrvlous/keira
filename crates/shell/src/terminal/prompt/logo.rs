// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Terminal banner and boot logo display.

use keira_io::vga;

/// Print the Keira ASCII boot banner and kernel version string.
pub fn print_logo() {
    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    vga::print_str("Keira Kernel ");
    vga::print_str(env!("CARGO_PKG_VERSION"));
    vga::print_str("-keira-1 (tty1)\n\n");
}
