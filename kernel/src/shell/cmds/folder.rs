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
//! Implementation of the 'folder' shell command.

use crate::io::vga;
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
                vga::print_str("Usage: folder <foldername>\n\n");
                vga::print_str("Description:\n  Create a new directory in the active directory on FAT16 storage.\n\n");
                vga::print_str("Options:\n  -h, --help    Show this help message and exit\n\n");
                vga::print_str("Examples:\n  folder projects\n");
                return;
            }
            Some(s) => s,
            None => {
                vga::print_str("Usage: folder <foldername>\n");
                return;
            }
        };
        unsafe {
            match crate::fs::fat::create_dir(arg) {
                Ok(_) => {
                    vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                    vga::print_str("Success: Folder created.\n");
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
