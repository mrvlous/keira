// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

#![allow(unused_variables, unused_unsafe)]

//!
//! Implementation of the 'edit' / 'nano' terminal text editor shell command.

use crate::args::CliArgs;
use crate::editor::editor_start;
use crate::executor::*;
use keira_io::vga;

pub fn run(parts: &mut core::str::SplitWhitespace) {
    let args = CliArgs::parse(parts);

    if args.has_flag('h', "help") {
        vga::set_color(vga::Color::White, vga::Color::Black);
        vga::print_str("Usage: edit <filename>  or  nano <filename>\n\n");
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        vga::print_str("Fullscreen GNU nano-style interactive text editor with syntax\nhighlighting, multi-line scrolling, cut/paste, search, and telemetry.\n\n");

        vga::set_color(vga::Color::White, vga::Color::Black);
        vga::print_str("Keyboard Shortcuts:\n");
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        vga::print_str("  ^G (Ctrl+G)   Get Help       Display help manual\n");
        vga::print_str("  ^O (Ctrl+O)   WriteOut       Save buffer to FAT16 storage\n");
        vga::print_str("  ^W (Ctrl+W)   Where Is       Search for string in text\n");
        vga::print_str("  ^K (Ctrl+K)   Cut Text       Cut line to clipboard buffer\n");
        vga::print_str("  ^U (Ctrl+U)   Paste Text     Paste line from clipboard buffer\n");
        vga::print_str("  ^C (Ctrl+C)   Cur Pos        Report line, col, and char metrics\n");
        vga::print_str("  ^R (Ctrl+R)   Read File      Reload original file from disk\n");
        vga::print_str("  ^X (Ctrl+X)   Exit           Exit editor (prompts if modified)\n\n");

        vga::set_color(vga::Color::White, vga::Color::Black);
        vga::print_str("Examples:\n");
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        vga::print_str("  edit /apps/main.c\n");
        vga::print_str("  nano script.sh\n");
        return;
    }

    unsafe {
        if !check_write_permission() {
            vga::set_color(vga::Color::LightRed, vga::Color::Black);
            vga::print_str("Permission denied: Non-admin users cannot write outside their home directory. Use 'please' to run as admin.\n");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            return;
        }

        let filename = match args.first_positional() {
            Some(f) => f,
            None => {
                vga::set_color(vga::Color::White, vga::Color::Black);
                vga::print_str("Usage: edit <filename>  or  nano <filename>\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                return;
            }
        };

        if let Err(e) = editor_start(filename) {
            vga::set_color(vga::Color::LightRed, vga::Color::Black);
            vga::print_str("Error starting editor: ");
            vga::print_str(e);
            vga::print_str("\n");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        }
    }
}
