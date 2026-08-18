#![allow(unused_variables, unused_unsafe)]
// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Keira Kernel: Shell Command 'kfence'
//!
//! Query Kernel Electric-Fence (KFENCE) memory out-of-bounds guards (Syscall 58).

use crate::io::vga;

pub fn run(parts: &mut core::str::SplitWhitespace) {
    if let Some("-h") | Some("--help") = parts.next() {
        unsafe {
            vga::print_str("Usage: kfence [status|sample]\n\n");
            vga::print_str("Description:\n  Query Kernel Electric-Fence (KFENCE) sampling rate and guard pages (Syscall 58).\n\n");
            vga::print_str("Options:\n  -h, --help    Show this help message and exit\n");
        }
        return;
    }

    unsafe {
        let _ = crate::mem::kfence::sys_kfence(100, 0);
    }
}
