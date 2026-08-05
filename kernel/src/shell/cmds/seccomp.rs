#![allow(unused_variables, unused_unsafe)]
// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Keira Kernel: Shell Command 'seccomp'
//!
//! View active seccomp BPF syscall filter sandbox status (Syscall 52).

use crate::io::vga;

pub fn run(parts: &mut core::str::SplitWhitespace) {
    if let Some("-h") | Some("--help") = parts.next() {
        unsafe {
            vga::print_str("Usage: seccomp [status]\n\n");
            vga::print_str("Description:\n  Inspect active seccomp BPF system call filter sandbox status (Syscall 52).\n\n");
            vga::print_str("Options:\n  -h, --help    Show this help message and exit\n");
        }
        return;
    }

    unsafe {
        vga::set_color(vga::Color::LightCyan, vga::Color::Black);
        vga::print_str("Seccomp BPF Syscall Filtering Sandbox Status (Syscall 52)\n");
        let _ = crate::task::seccomp::sys_seccomp(0, 0, 0);
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    }
}
