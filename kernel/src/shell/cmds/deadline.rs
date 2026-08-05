#![allow(unused_variables, unused_unsafe)]
// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Keira Kernel: Shell Command 'deadline'
//!
//! Inspect Sched_Deadline EDF real-time task scheduler policy (Syscall 64).

use crate::io::vga;

pub fn run(parts: &mut core::str::SplitWhitespace) {
    if let Some("-h") | Some("--help") = parts.next() {
        unsafe {
            vga::print_str("Usage: deadline [status]\n\n");
            vga::print_str("Description:\n  Inspect POSIX Sched_Deadline EDF hard real-time scheduler policy status (Syscall 64).\n\n");
            vga::print_str("Options:\n  -h, --help    Show this help message and exit\n");
        }
        return;
    }

    unsafe {
        vga::set_color(vga::Color::LightCyan, vga::Color::Black);
        vga::print_str("POSIX Sched_Deadline EDF Hard Real-Time Scheduler (Syscall 64)\n");
        let _ = crate::task::deadline::sys_sched_setattr(1, 0, 0);
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    }
}
