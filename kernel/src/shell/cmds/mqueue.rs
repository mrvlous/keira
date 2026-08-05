#![allow(unused_variables, unused_unsafe)]
// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Keira Kernel: Shell Command 'mqueue'
//!
//! Inspect in-kernel POSIX Message Queue IPC status (Syscall 58).

use crate::io::vga;

pub fn run(parts: &mut core::str::SplitWhitespace) {
    if let Some("-h") | Some("--help") = parts.next() {
        unsafe {
            vga::print_str("Usage: mqueue [status]\n\n");
            vga::print_str("Description:\n  Inspect in-kernel POSIX Message Queue IPC subsystem status (Syscall 58).\n\n");
            vga::print_str("Options:\n  -h, --help    Show this help message and exit\n");
        }
        return;
    }

    unsafe {
        vga::set_color(vga::Color::LightCyan, vga::Color::Black);
        vga::print_str("POSIX Message Queue IPC Subsystem Status (Syscall 58)\n");
        let _ = crate::ipc::mqueue::sys_mq_open(core::ptr::null(), 0, 0);
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    }
}
