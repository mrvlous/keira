#![allow(unused_variables, unused_unsafe)]
// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Keira Kernel: Shell Command 'swap'
//!
//! View active virtual memory swap space partitions (Syscall 53 & 54).

use crate::io::vga;

pub fn run(parts: &mut core::str::SplitWhitespace) {
    if let Some("-h") | Some("--help") = parts.next() {
        unsafe {
            vga::print_str("Usage: swap [status|on|off]\n\n");
            vga::print_str("Description:\n  Inspect active virtual memory swap space partition status (Syscall 53 & 54).\n\n");
            vga::print_str("Options:\n  -h, --help    Show this help message and exit\n");
        }
        return;
    }

    unsafe {
        vga::set_color(vga::Color::LightCyan, vga::Color::Black);
        vga::print_str("Virtual Memory Swap Space Partition Status (Syscall 53 & 54)\n");
        let _ = crate::mem::swap::sys_swapon(core::ptr::null(), 0);
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    }
}
