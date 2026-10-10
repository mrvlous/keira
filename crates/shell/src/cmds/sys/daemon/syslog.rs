// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! View kernel and system event log records.

use crate::args::CliArgs;
use keira_io::vga;

pub fn run(parts: &mut core::str::SplitWhitespace) {
    let args = CliArgs::parse(parts);

    if args.has_flag('h', "help") {
        vga::set_color(vga::Color::White, vga::Color::Black);
        vga::print_str("Usage: syslog [OPTIONS]\n\n");
        vga::print_str("Description:\n  Display system and kernel event log records.\n\n");
        vga::print_str("Options:\n");
        vga::print_str("  -b, --boot  Display system boot record instead of runtime syslog\n");
        vga::print_str("  -h, --help  Show this help message and exit\n");
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        return;
    }

    let path = if args.has_flag('b', "boot") {
        "/var/log/boot.log"
    } else {
        "/var/log/system.log"
    };

    let mut buf = [0u8; 1024];
    match keira_fs::vfs::read_file(path, &mut buf) {
        Ok(len) if len > 0 => {
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            if let Ok(s) = core::str::from_utf8(&buf[..len]) {
                vga::print_str(s);
                if !s.ends_with('\n') {
                    vga::print_str("\n");
                }
            }
        }
        _ => {
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            vga::print_str("syslog: no active log records available\n");
        }
    }
}
