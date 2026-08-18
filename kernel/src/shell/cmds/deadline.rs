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
//! Configure SCHED_DEADLINE Earliest Deadline First (EDF) real-time policy (Syscall 65).

use crate::io::vga;

pub fn run(parts: &mut core::str::SplitWhitespace) {
    if let Some("-h") | Some("--help") = parts.next() {
        unsafe {
            vga::print_str("Usage: deadline [status|set <pid>]\n\n");
            vga::print_str("Description:\n  Configure SCHED_DEADLINE Earliest Deadline First (EDF) real-time scheduler policy (Syscall 65).\n\n");
            vga::print_str("Options:\n  -h, --help    Show this help message and exit\n");
        }
        return;
    }

    unsafe {
        let _ = crate::task::deadline::sys_sched_setattr(1, 0, 0);
    }
}
