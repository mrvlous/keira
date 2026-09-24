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

use crate::executor::dispatch::auth::get_current_user_home;
use crate::state::session::{
    CURRENT_USER, CURRENT_USER_LEN, HOSTNAME, HOSTNAME_LEN, IN_EDITOR_MODE, PROMPT_COL, PROMPT_ROW,
    SHELL_PATH, SHELL_PATH_LEN,
};

/// Print the shell prompt and record cursor position in authentic Linux console style.
pub fn print_prompt() {
    unsafe {
        let in_ed = &raw const IN_EDITOR_MODE;
        if *in_ed {
            return;
        }

        vga::set_color(vga::Color::White, vga::Color::Black);
        let ulen = core::cmp::min(CURRENT_USER_LEN, 16);
        if let Ok(user_str) = core::str::from_utf8(&CURRENT_USER[..ulen]) {
            vga::print_str(user_str);
        } else {
            vga::print_str("default");
        }

        vga::print_str("@");
        let hlen = core::cmp::min(HOSTNAME_LEN, 32);
        if let Ok(hostname_str) = core::str::from_utf8(&HOSTNAME[..hlen]) {
            vga::print_str(hostname_str);
        } else {
            vga::print_str("keira");
        }
        vga::print_str(":");

        let plen = core::cmp::min(SHELL_PATH_LEN, 80);
        let current_path = core::str::from_utf8(&SHELL_PATH[..plen]).unwrap_or_default();
        let home_path = get_current_user_home();

        if current_path.is_empty() {
            vga::print_str("/");
        } else if current_path == home_path {
            vga::putchar(b'~');
        } else if current_path.starts_with(home_path)
            && current_path.len() > home_path.len()
            && current_path.as_bytes()[home_path.len()] == b'/'
        {
            vga::putchar(b'~');
            vga::print_str(&current_path[home_path.len()..]);
        } else {
            vga::print_str("/");
            vga::print_str(current_path);
        }

        vga::print_str("$ ");
        vga::set_color(vga::Color::White, vga::Color::Black);

        PROMPT_COL = vga::get_cursor_col();
        PROMPT_ROW = vga::get_cursor_row();
    }
}
