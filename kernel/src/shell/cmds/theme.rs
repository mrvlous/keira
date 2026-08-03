#![allow(unused_variables, unused_unsafe)]
// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Keira Kernel: Shell Command 'theme'
//!
//! Implementation of the dynamic color theme selection command.

use crate::io::vga;
use crate::shell::state::*;

pub fn run(parts: &mut core::str::SplitWhitespace) {
    let theme_name = match parts.next() {
        Some("-h") | Some("--help") => {
            vga::print_str("Usage: theme [retro|matrix|arch|classic|dracula]\n\n");
            vga::print_str(
                "Description:\n  Switch active VGA terminal color scheme palette in real-time.\n\n",
            );
            vga::print_str("Options:\n  -h, --help    Show this help message and exit\n\n");
            vga::print_str("Available Themes:\n  retro    Monochrome green phosphor terminal\n  matrix   Matrix style lime & dark green console\n  arch     Cybernetic blue on black theme\n  classic  Standard retro white on black\n  dracula  Premium gothic dark purple & magenta\n");
            return;
        }
        Some(name) => name,
        None => {
            vga::print_str("Usage: theme [retro|matrix|arch|classic|dracula]\n");
            vga::print_str("Current available themes:\n");
            vga::print_str("  retro   - Monochrome green phosphor terminal\n");
            vga::print_str("  matrix  - Matrix style lime & dark green console\n");
            vga::print_str("  arch    - Cybernetic blue on black theme\n");
            vga::print_str("  classic - Standard retro white on black\n");
            vga::print_str("  dracula - Premium gothic dark purple & magenta\n");
            return;
        }
    };

    unsafe {
        let mut valid = true;
        match theme_name {
            "retro" => {
                CURRENT_THEME = ShellTheme {
                    user: vga::Color::Green,
                    host: vga::Color::Green,
                    path: vga::Color::LightGreen,
                    symbol: vga::Color::LightGreen,
                    text_fg: vga::Color::LightGreen,
                    text_bg: vga::Color::Black,
                };
            }
            "matrix" => {
                CURRENT_THEME = ShellTheme {
                    user: vga::Color::LightGreen,
                    host: vga::Color::LightGreen,
                    path: vga::Color::Green,
                    symbol: vga::Color::Green,
                    text_fg: vga::Color::Green,
                    text_bg: vga::Color::Black,
                };
            }
            "arch" => {
                CURRENT_THEME = ShellTheme {
                    user: vga::Color::LightBlue,
                    host: vga::Color::LightCyan,
                    path: vga::Color::LightBlue,
                    symbol: vga::Color::Cyan,
                    text_fg: vga::Color::LightCyan,
                    text_bg: vga::Color::Black,
                };
            }
            "classic" => {
                CURRENT_THEME = ShellTheme {
                    user: vga::Color::LightRed,
                    host: vga::Color::LightCyan,
                    path: vga::Color::LightBlue,
                    symbol: vga::Color::LightGreen,
                    text_fg: vga::Color::LightGrey,
                    text_bg: vga::Color::Black,
                };
            }
            "dracula" => {
                CURRENT_THEME = ShellTheme {
                    user: vga::Color::LightMagenta,
                    host: vga::Color::LightBlue,
                    path: vga::Color::LightMagenta,
                    symbol: vga::Color::LightGreen,
                    text_fg: vga::Color::White,
                    text_bg: vga::Color::Black,
                };
            }
            _ => {
                vga::print_str("theme error: unknown theme. Choose from classic, retro, matrix, arch, dracula.\n");
                valid = false;
            }
        }

        if valid {
            // Set the new text color attribute in the driver
            vga::set_color(CURRENT_THEME.text_fg, CURRENT_THEME.text_bg);
            // Re-initialize the screen to clear it using the new theme background color
            vga::init();
            vga::print_str("Theme changed successfully!\n");
        }
    }
}
