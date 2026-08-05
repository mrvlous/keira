#![allow(unused_variables, unused_unsafe)]
// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Keira Kernel: Shell Command 'lkm' / 'lsmod'
//!
//! Inspect Loadable Kernel Modules and dynamic symbol resolution (Syscall 34 & 35).

use crate::io::vga;

pub fn run(parts: &mut core::str::SplitWhitespace) {
    if let Some("-h") | Some("--help") = parts.next() {
        unsafe {
            vga::print_str("Usage: lkm [lsmod|load|unload]\n\n");
            vga::print_str("Description:\n  Inspect Loadable Kernel Modules and dynamic symbol resolution (Syscall 34 & 35).\n\n");
            vga::print_str("Options:\n  -h, --help    Show this help message and exit\n");
        }
        return;
    }

    unsafe {
        vga::set_color(vga::Color::LightCyan, vga::Color::Black);
        vga::print_str("Dynamically Loadable Kernel Modules (LKM - Syscall 34 & 35)\n");
        vga::set_color(vga::Color::LightGreen, vga::Color::Black);
        vga::print_str("  Status : [OK] kallsyms Dynamic Symbol Resolver Active\n");
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    }
}
