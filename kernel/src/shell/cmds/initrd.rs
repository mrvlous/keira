#![allow(unused_variables, unused_unsafe)]
// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Keira Kernel: Shell Command 'initrd'
//!
//! Implementation of the 'initrd' shell command.

pub fn run(parts: &mut core::str::SplitWhitespace) {
    if let Some("-h") | Some("--help") = parts.next() {
        unsafe {
            crate::io::vga::print_str("Usage: initrd\n\n");
            crate::io::vga::print_str("Description:\n  List all preloaded files and sizes stored in the read-only Initrd TAR RAM disk.\n\n");
            crate::io::vga::print_str(
                "Options:\n  -h, --help    Show this help message and exit\n",
            );
        }
        return;
    }

    unsafe {
        crate::fs::tar::list_files();
    }
}
