#![allow(unused_variables, unused_unsafe)]
// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Keira Kernel: Shell Command 'runtime'
//!
//! Implementation of the 'runtime' shell command.

use crate::io::vga;
use crate::shell::executor::*;

pub fn run(parts: &mut core::str::SplitWhitespace) {
    unsafe {
        if let Some("-h") | Some("--help") = parts.next() {
            vga::print_str("Usage: runtime\n\n");
            vga::print_str("Description:\n  Display high-precision system uptime since kernel boot in hours, minutes, seconds, and milliseconds.\n\n");
            vga::print_str("Options:\n  -h, --help    Show this help message and exit\n");
            return;
        }

        let ms = unsafe { get_uptime_ms() };
        let hours = ms / 3600000;
        let minutes = (ms % 3600000) / 60000;
        let seconds = (ms % 60000) / 1000;
        let millis = ms % 1000;
        vga::set_color(vga::Color::LightBlue, vga::Color::Black);
        vga::print_str("System runtime: ");
        vga::set_color(vga::Color::White, vga::Color::Black);
        vga::print_u64(hours);
        vga::print_str("h ");
        vga::print_u64(minutes);
        vga::print_str("m ");
        vga::print_u64(seconds);
        vga::print_str("s ");
        vga::print_u64(millis);
        vga::print_str("ms\n");
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    }
}
