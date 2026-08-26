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
//! Flush dirty filesystem block cache pages to physical storage device.

use keira_fs::fat;
use keira_io::vga;

pub fn run(_parts: &mut core::str::SplitWhitespace) {
    unsafe {
        vga::set_color(vga::Color::White, vga::Color::Black);
        vga::print_str("Flushing dirty filesystem sectors to storage...\n");
        match fat::flush_dirty_sectors() {
            Ok(count) => {
                vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                vga::print_str("[OK] Flushed ");
                vga::print_u64(count as u64);
                vga::print_str(" block sectors to disk.\n");
            }
            Err(e) => {
                vga::set_color(vga::Color::LightRed, vga::Color::Black);
                vga::print_str("Sync Error: ");
                vga::print_str(e);
                vga::print_str("\n");
            }
        }
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    }
}
