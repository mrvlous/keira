// SPDX-License-Identifier: GPL-2.0-only
/*
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra <mrvlous@proton.me>
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 */

//! View kernel diagnostic ring buffer records.

use crate::args::CliArgs;
use keira_io::vga;

pub fn run(parts: &mut core::str::SplitWhitespace) {
    let args = CliArgs::parse(parts);

    if args.has_flag('h', "help") {
        vga::set_color(vga::Color::White, vga::Color::Black);
        vga::print_str("Usage: syslog [OPTIONS]\n\n");
        vga::print_str("Description:\n  Display live kernel diagnostic ring buffer records.\n\n");
        vga::print_str("Options:\n");
        vga::print_str("  -c, --clear  Clear kernel diagnostic ring buffer\n");
        vga::print_str("  -h, --help   Show this help message and exit\n");
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        return;
    }

    if args.has_flag('c', "clear") {
        keira_core::klog_clear();
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        vga::print_str("syslog: kernel ring buffer cleared\n");
        return;
    }

    let mut total_bytes = 0;
    keira_core::klog_get_records(keira_core::KLOG_BUFFER_SIZE, |chunk| {
        if let Ok(s) = core::str::from_utf8(chunk) {
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            vga::print_str(s);
            total_bytes += chunk.len();
        }
        Ok(())
    });

    if total_bytes == 0 {
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        vga::print_str("syslog: no active log records available\n");
    }
}
