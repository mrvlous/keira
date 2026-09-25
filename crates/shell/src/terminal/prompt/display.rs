// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Shell command prompt formatting and cursor telemetry.

use keira_io::vga;

use crate::state::session::{
    HOSTNAME, HOSTNAME_LEN, IN_EDITOR_MODE, PROMPT_COL, PROMPT_ROW, SHELL_PATH, SHELL_PATH_LEN,
};

/// Print the kernel monitor shell prompt and record cursor position.
pub fn print_prompt() {
    unsafe {
        let in_ed = &raw const IN_EDITOR_MODE;
        if *in_ed {
            return;
        }

        vga::set_color(vga::Color::White, vga::Color::Black);
        let hlen = core::cmp::min(HOSTNAME_LEN, 32);
        if let Ok(hostname_str) = core::str::from_utf8(&HOSTNAME[..hlen]) {
            vga::print_str(hostname_str);
        } else {
            vga::print_str("keira");
        }
        vga::print_str(":");

        let plen = core::cmp::min(SHELL_PATH_LEN, 80);
        let current_path = core::str::from_utf8(&SHELL_PATH[..plen]).unwrap_or_default();

        if current_path.is_empty() {
            vga::print_str("/");
        } else {
            vga::print_str("/");
            vga::print_str(current_path);
        }

        vga::print_str("# ");
        vga::set_color(vga::Color::White, vga::Color::Black);

        PROMPT_COL = vga::get_cursor_col();
        PROMPT_ROW = vga::get_cursor_row();
    }
}
