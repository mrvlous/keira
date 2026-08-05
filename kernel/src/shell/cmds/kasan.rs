#![allow(unused_variables, unused_unsafe)]
// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Keira Kernel: Shell Command 'kasan'
//!
//! Query KASAN shadow memory diagnostic tracer status (Syscall 57).

use crate::io::vga;

pub fn run(parts: &mut core::str::SplitWhitespace) {
    if let Some("-h") | Some("--help") = parts.next() {
        unsafe {
            vga::print_str("Usage: kasan [status]\n\n");
            vga::print_str("Description:\n  Query KASAN shadow memory diagnostic tracer status (Syscall 57).\n\n");
            vga::print_str("Options:\n  -h, --help    Show this help message and exit\n");
        }
        return;
    }

    unsafe {
        vga::set_color(vga::Color::LightCyan, vga::Color::Black);
        vga::print_str("KASAN Shadow Memory Diagnostic Tracer Status (Syscall 57)\n");
        let _ = crate::mem::kasan::sys_kasan(0x100000, 4096);
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    }
}
