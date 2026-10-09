// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Implementation of the 'init' shell command to inspect and invoke userspace init (PID 1).

use crate::cmds::proc::task::run::run_user_program;
use keira_io::vga;

/// Execute the 'init' shell command to query or launch the canonical userspace init process.
pub fn run(parts: &mut core::str::SplitWhitespace) {
    if let Some("-h") | Some("--help") = parts.next() {
        vga::set_color(vga::Color::White, vga::Color::Black);
        vga::print_str("Usage: init [-r|--run] [-h|--help]\n\n");
        vga::print_str("Description:\n  Query status or invoke the canonical userspace init process (PID 1).\n\n");
        vga::print_str("Options:\n");
        vga::print_str("  -r, --run      Launch userspace init binary (/bin/init.elf) in Ring 3\n");
        vga::print_str("  -h, --help     Show this help message and exit\n");
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        return;
    }

    vga::set_color(vga::Color::White, vga::Color::Black);
    vga::print_str("Keira Canonical Userspace Init Status (PID 1)\n");
    vga::print_str("---------------------------------------------\n");
    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    vga::print_str("  Canonical Binary : /bin/init.elf (or /bin/init)\n");
    vga::print_str("  Target Privilege : Ring 3 (User Mode)\n");
    vga::print_str("  Mount Contract   : /dev, /proc, /sys, /etc, /tmp\n");

    let init_exists = keira_fs::exists("/bin/init.elf") || keira_fs::exists("/bin/init");
    vga::print_str("  Binary Status    : ");
    if init_exists {
        vga::set_color(vga::Color::LightGreen, vga::Color::Black);
        vga::print_str("Present on Root Filesystem\n");
    } else {
        vga::set_color(vga::Color::LightRed, vga::Color::Black);
        vga::print_str("Missing (Using Kernel Supervisor Fallback)\n");
    }
    vga::set_color(vga::Color::LightGrey, vga::Color::Black);

    if init_exists {
        vga::print_str("Executing userspace init in Ring 3...\n");
        let path = if keira_fs::exists("/bin/init.elf") {
            "/bin/init.elf"
        } else {
            "/bin/init"
        };
        unsafe {
            if let Err(e) = run_user_program(path, &[path, "-v"]) {
                vga::set_color(vga::Color::LightRed, vga::Color::Black);
                vga::print_str("Init Execution Failed: ");
                vga::print_str(e);
                vga::print_str("\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            }
        }
    }
}
