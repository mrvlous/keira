// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Implementation of the 'help' shell command.

use keira_io::vga;

pub fn run(_parts: &mut core::str::SplitWhitespace) {
    {
        let bg = vga::Color::Black;
        vga::set_color(vga::Color::White, bg);
        vga::print_str("Keira Kernel Diagnostic & Emergency Debugger\n");
        vga::set_color(vga::Color::LightGrey, bg);
        vga::print_str(
            "Use '<command> --help' to view options and detailed usage for any command.\n",
        );
        vga::print_str("Standard UNIX utilities reside in /bin and run in Ring 3 userspace.\n\n");

        vga::set_color(vga::Color::White, bg);
        vga::print_str("Hardware & System Diagnostics:\n");
        vga::set_color(vga::Color::LightGrey, bg);
        vga::print_str("  system      cpu         smp         memory      devices     drivers\n\n");

        vga::set_color(vga::Color::White, bg);
        vga::print_str("Emergency Control & Recovery:\n");
        vga::set_color(vga::Color::LightGrey, bg);
        vga::print_str("  disk        sync        syslog      unwind      watchpoint\n\n");

        vga::set_color(vga::Color::White, bg);
        vga::print_str("Execution & Power Management:\n");
        vga::set_color(vga::Color::LightGrey, bg);
        vga::print_str("  run         reset       power       help\n");
    }
}
