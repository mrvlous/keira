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
//! Inspect native EXT4 filesystem partition superblock & inodes.

use keira_io::vga;

pub fn run(parts: &mut core::str::SplitWhitespace) {
    if let Some("-h") | Some("--help") = parts.next() {
        unsafe {
            vga::print_str("Usage: ext4 [info|inodes]\n\n");
            vga::print_str("Description:\n  Inspect native Linux EXT4 / EXT2 filesystem partition superblock & inodes.\n\n");
            vga::print_str("Options:\n  -h, --help    Show this help message and exit\n");
        }
        return;
    }

    unsafe {
        vga::set_color(vga::Color::White, vga::Color::Black);
        vga::print_str("Native Linux EXT4 Filesystem Driver Status\n");
        let _ = keira_fs::ext4::init();
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    }
}
