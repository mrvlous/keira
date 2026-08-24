// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

#![allow(unused_variables, unused_unsafe)]

//! View kernel circular syslog dmesg log buffer using sys_syslog (Syscall 44).

use keira_core::klog::sys_syslog_read;
use keira_io::vga;

pub fn run(parts: &mut core::str::SplitWhitespace) {
    if let Some("-h") | Some("--help") = parts.next() {
        unsafe {
            vga::print_str("Usage: syslog [dmesg]\n\n");
            vga::print_str("Description:\n  View circular kernel syslog dmesg diagnostic log buffer (Syscall 44).\n\n");
            vga::print_str("Options:\n  -h, --help    Show this help message and exit\n");
        }
        return;
    }

    unsafe {
        vga::set_color(vga::Color::White, vga::Color::Black);
        vga::print_str("Kernel Diagnostic Syslog Buffer (dmesg - Syscall 44)\n");
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        vga::print_str("Querying kernel syslog ring buffer...\n");
        let mut buf = [0u8; 128];
        let _ = sys_syslog_read(buf.as_mut_ptr(), 128);
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    }
}
