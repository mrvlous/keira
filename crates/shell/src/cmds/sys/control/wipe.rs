// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Clear and reset the VGA text console display buffer.

use keira_io::vga;

/// Executes the 'wipe' command to clear the console display and reset the cursor.
pub fn run(parts: &mut core::str::SplitWhitespace) {
    if let Some("-h") | Some("--help") = parts.next() {
        vga::set_color(vga::Color::White, vga::Color::Black);
        vga::print_str("Usage: wipe [-h]\n\n");
        vga::print_str("Description:\n  Clear and reset the VGA text console display buffer.\n\n");
        vga::print_str("Options:\n  -h, --help    Show this help message and exit\n");
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        return;
    }

    vga::init();
    vga::set_cursor_pos(0, 0);
}
