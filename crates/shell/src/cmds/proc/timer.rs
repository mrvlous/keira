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
//! Inspect POSIX high-resolution nanosecond interval timers (Syscall 45 & 46).

use keira_io::vga;

pub fn run(parts: &mut core::str::SplitWhitespace) {
    if let Some("-h") | Some("--help") = parts.next() {
        unsafe {
            vga::print_str("Usage: timer [status]\n\n");
            vga::print_str("Description:\n  Inspect POSIX high-resolution nanosecond interval timers (Syscall 45 & 46).\n\n");
            vga::print_str("Options:\n  -h, --help    Show this help message and exit\n");
        }
        return;
    }

    unsafe {
        let mut tid: u64 = 0;
        let _ = keira_arch::timer::sys_timer_create(0, &mut tid);
    }
}
