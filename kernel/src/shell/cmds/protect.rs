// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Keira Kernel: Shell Command 'protect'
//!
//! Implementation of the native 'protect' shell command to toggle file attributes
//! (e.g. 'protect /data/file.txt readonly' or 'protect /data/file.txt readwrite').

use crate::fs::fat;
use crate::io::vga;

pub fn run(parts: &mut core::str::SplitWhitespace) {
    let path = parts.next();
    let mode = parts.next();

    if let Some("-h") | Some("--help") = path {
        vga::print_str("Usage: protect <file_path> <readonly|readwrite>\n\n");
        vga::print_str("Description:\n  Toggle read-only (0x01) or read-write attribute protection on a FAT16 file entry.\n\n");
        vga::print_str("Options:\n  -h, --help    Show this help message and exit\n\n");
        vga::print_str("Examples:\n  protect note.txt readonly\n  protect note.txt readwrite\n");
        return;
    }

    match (path, mode) {
        (Some(p), Some(m)) => unsafe {
            let (dir_cluster, name) = match fat::resolve_path(p) {
                Ok(res) => res,
                Err(_) => {
                    vga::print_str("Protect Error: Invalid file path\n");
                    return;
                }
            };

            let _entry = match fat::find_entry(name, dir_cluster) {
                Ok(e) => e,
                Err(_) => {
                    vga::print_str("Protect Error: File not found\n");
                    return;
                }
            };

            match m {
                "readonly" => {
                    vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                    vga::print_str("[OK] File ");
                    vga::print_str(p);
                    vga::print_str(" protection set to READ-ONLY\n");
                    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                }
                "readwrite" => {
                    vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                    vga::print_str("[OK] File ");
                    vga::print_str(p);
                    vga::print_str(" protection set to READ-WRITE\n");
                    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                }
                _ => {
                    vga::print_str("Usage: protect <file_path> <readonly|readwrite>\n");
                }
            }
        },
        _ => {
            vga::print_str("Usage: protect <file_path> <readonly|readwrite>\n");
        }
    }
}
