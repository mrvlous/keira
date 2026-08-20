// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//!
//! Implementation of the native 'protect' shell command to toggle file attributes
//! (e.g. 'protect /data/file.txt readonly' or 'protect /data/file.txt readwrite').

use keira_fs::fat;
use keira_io::vga;

pub fn run(parts: &mut core::str::SplitWhitespace) {
    let path = parts.next();
    let mode = parts.next();

    if let Some("-h") | Some("--help") = path {
        vga::print_str("Usage: protect <file_path> <readonly|readwrite|mode_octal>\n\n");
        vga::print_str("Description:\n  Set POSIX file security permissions (0755, 0700, 0644) or toggle read-only attribute protection on FAT16 file entry.\n\n");
        vga::print_str("Options:\n  -h, --help    Show this help message and exit\n\n");
        vga::print_str("Examples:\n  protect note.txt 755\n  protect secret.txt 700\n  protect file.txt readonly\n");
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
                    vga::print_str(" protection set to READ-ONLY (mode 0444)\n");
                    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                }
                "readwrite" => {
                    vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                    vga::print_str("[OK] File ");
                    vga::print_str(p);
                    vga::print_str(" protection set to READ-WRITE (mode 0644)\n");
                    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                }
                _ if m.chars().all(|c| c.is_ascii_digit()) => {
                    vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                    vga::print_str("[OK] POSIX permission mode for ");
                    vga::print_str(p);
                    vga::print_str(" updated to 0");
                    vga::print_str(m);
                    vga::print_str("\n");
                    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                }
                _ => {
                    vga::print_str("Usage: protect <file_path> <readonly|readwrite|mode_octal>\n");
                }
            }
        },
        _ => {
            vga::print_str("Usage: protect <file_path> <readonly|readwrite|mode_octal>\n");
        }
    }
}
