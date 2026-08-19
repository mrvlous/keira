// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

#![allow(unused_variables, unused_unsafe, static_mut_refs)]

//!
//! Implementation of the 'view' shell command.

use crate::io::vga;

static mut VIEW_BUF: [u8; 8192] = [0u8; 8192];

pub fn run(parts: &mut core::str::SplitWhitespace) {
    unsafe {
        let arg = match parts.next() {
            Some("-h") | Some("--help") => {
                vga::print_str("Usage: view <filename>\n\n");
                vga::print_str("Description:\n  Read file contents from FAT16 storage and print raw UTF-8 text string to VGA console stream.\n\n");
                vga::print_str("Options:\n  -h, --help    Show this help message and exit\n\n");
                vga::print_str("Examples:\n  view note.txt\n");
                return;
            }
            Some(s) => s,
            None => {
                vga::print_str("Usage: view <filename>\n");
                return;
            }
        };

        match crate::fs::vfs::read_file(arg, &mut VIEW_BUF) {
            Ok(len) => {
                vga::set_color(vga::Color::White, vga::Color::Black);
                let slice = &VIEW_BUF[..len];
                if let Ok(text) = core::str::from_utf8(slice) {
                    vga::print_str(text);
                    vga::print_str("\n");
                } else {
                    for &b in slice {
                        if b == b'\n' || b == b'\r' || b == b'\t' || (b >= 0x20 && b <= 0x7E) {
                            let s = [b];
                            if let Ok(ch) = core::str::from_utf8(&s) {
                                vga::print_str(ch);
                            }
                        } else {
                            vga::print_str(".");
                        }
                    }
                    vga::print_str("\n");
                }
            }
            Err(e) => {
                vga::set_color(vga::Color::LightRed, vga::Color::Black);
                vga::print_str("Error viewing file: ");
                vga::print_str(e);
                vga::print_str("\n");
            }
        }
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    }
}
