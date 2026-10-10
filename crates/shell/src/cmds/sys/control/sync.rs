// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Flush dirty filesystem block cache pages to physical storage device.

use keira_fs::fat;
use keira_io::vga;

pub fn run(parts: &mut core::str::SplitWhitespace) {
    if let Some("-h") | Some("--help") = parts.next() {
        vga::set_color(vga::Color::White, vga::Color::Black);
        vga::print_str("Usage: sync [-h]\n\n");
        vga::print_str(
            "Description:\n  Flush dirty filesystem block cache pages to physical storage.\n\n",
        );
        vga::print_str("Options:\n  -h, --help    Show this help message and exit\n");
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        return;
    }

    unsafe {
        match fat::flush_dirty_sectors() {
            Ok(count) => {
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                vga::print_str("sync: flushed ");
                vga::print_u64(count as u64);
                vga::print_str(" dirty sectors to storage\n");
            }
            Err(e) => {
                vga::set_color(vga::Color::LightRed, vga::Color::Black);
                vga::print_str("sync: error: ");
                vga::print_str(e);
                vga::print_str("\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            }
        }
    }
}
