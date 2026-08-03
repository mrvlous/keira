#![allow(unused_variables, unused_unsafe)]
// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Keira Kernel: Shell Command 'edit'
//!
//! Implementation of the 'edit' shell command.

use crate::io::vga;
use crate::shell::editor::editor_start;
use crate::shell::executor::*;

pub fn run(parts: &mut core::str::SplitWhitespace) {
    unsafe {
        unsafe {
            if !check_write_permission() {
                vga::set_color(vga::Color::LightRed, vga::Color::Black);
                vga::print_str("Permission denied: Non-admin users cannot write outside their home directory. Use 'please' to run as admin.\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                return;
            }
        }
        let arg = match parts.next() {
            Some("-h") | Some("--help") => {
                vga::print_str("Usage: edit <filename>\n\n");
                vga::print_str("Description:\n  Launch the interactive 128-line VGA text editor with syntax highlighting, search, and vertical scrolling.\n\n");
                vga::print_str("Options:\n  -h, --help    Show this help message and exit\n\n");
                vga::print_str("Examples:\n  edit main.c\n");
                return;
            }
            Some(s) => s,
            None => {
                vga::print_str("Usage: edit <filename>\n");
                return;
            }
        };
        unsafe {
            if let Err(e) = editor_start(arg) {
                vga::set_color(vga::Color::LightRed, vga::Color::Black);
                vga::print_str("Error starting editor: ");
                vga::print_str(e);
                vga::print_str("\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            }
        }
    }
}
