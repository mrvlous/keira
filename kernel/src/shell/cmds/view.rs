#![allow(unused_variables, unused_unsafe)]
// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Keira Kernel: Shell Command 'view'
//!
//! Implementation of the 'view' shell command.

use crate::io::vga;

pub fn run(parts: &mut core::str::SplitWhitespace) {
    unsafe {
        let arg = match parts.next() {
            Some(s) => s,
            None => {
                vga::print_str("Usage: view <filename>\n");
                return;
            }
        };
        let mut file_buf = [0u8; 8192];
        match crate::fs::vfs::read_file(arg, &mut file_buf) {
            Ok(len) => {
                if let Ok(text) = core::str::from_utf8(&file_buf[..len]) {
                    vga::print_str(text);
                    vga::print_str("\n");
                } else {
                    vga::print_str("Error: File contains invalid UTF-8 encoding\n");
                }
            }
            Err(e) => {
                vga::print_str("Error viewing file: ");
                vga::print_str(e);
                vga::print_str("\n");
            }
        }
    }
}
