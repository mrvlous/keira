// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Implementation of the 'create' shell command.

use keira_io::vga;

pub fn run(parts: &mut core::str::SplitWhitespace) {
    {
        let arg = match parts.next() {
            Some("-h") | Some("--help") => {
                vga::print_str("Usage: create <filename>\n\n");
                vga::print_str("Description:\n  Create a new empty file in the active directory on FAT16 storage.\n\n");
                vga::print_str("Options:\n  -h, --help    Show this help message and exit\n\n");
                vga::print_str("Examples:\n  create document.txt\n");
                return;
            }
            Some(s) => s,
            None => {
                vga::print_str("Usage: create <filename>\n");
                return;
            }
        };
        unsafe {
            match keira_fs::fat::create_file(arg) {
                Ok(_) => {
                    vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                    vga::print_str("Success: File created.\n");
                    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                }
                Err(e) => {
                    vga::set_color(vga::Color::LightRed, vga::Color::Black);
                    vga::print_str("Error: ");
                    vga::print_str(e);
                    vga::print_str("\n");
                    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                }
            }
        }
    }
}
