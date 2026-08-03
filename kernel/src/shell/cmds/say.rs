#![allow(unused_variables, unused_unsafe)]
// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Keira Kernel: Shell Command 'say'
//!
//! Implementation of the 'say' shell command.

use crate::io::vga;

pub fn run(parts: &mut core::str::SplitWhitespace) {
    let mut cloned = parts.clone();
    if let Some("-h") | Some("--help") = cloned.next() {
        unsafe {
            vga::print_str("Usage: say <message...>\n\n");
            vga::print_str("Description:\n  Echo back the input text message to the VGA console output stream.\n\n");
            vga::print_str("Options:\n  -h, --help    Show this help message and exit\n\n");
            vga::print_str("Examples:\n  say Hello Keira Kernel!\n");
        }
        return;
    }

    unsafe {
        let mut first = true;
        for part in parts {
            if !first {
                vga::print_str(" ");
            }
            vga::print_str(part);
            first = false;
        }
        vga::print_str("\n");
    }
}
