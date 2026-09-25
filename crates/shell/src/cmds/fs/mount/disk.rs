// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Implementation of the 'disk' shell command.

use keira_io::vga;

pub fn run(parts: &mut core::str::SplitWhitespace) {
    unsafe {
        if let Some("-h") | Some("--help") = parts.next() {
            vga::print_str("Usage: disk\n\n");
            vga::print_str("Description:\n  Display primary storage drive geometry, sector layout, and active FAT filesystem details.\n\n");
            vga::print_str("Options:\n  -h, --help    Show this help message and exit\n");
            return;
        }

        keira_fs::fat::print_disk_info();
    }
}
