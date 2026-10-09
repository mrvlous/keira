// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Implementation of the 'sh' shell command to launch the standalone Ring 3 userspace shell.

use crate::cmds::proc::task::run::run_user_program;
use keira_io::vga;

/// Execute the 'sh' command to launch the canonical standalone userspace shell.
pub fn run(parts: &mut core::str::SplitWhitespace) {
    let mut args_buf: [&str; 16] = [""; 16];
    let mut arg_count = 0;

    args_buf[0] = "sh";
    arg_count += 1;

    for part in parts.by_ref() {
        if arg_count == 1 && (part == "-h" || part == "--help") {
            vga::set_color(vga::Color::White, vga::Color::Black);
            vga::print_str("Usage: sh [-c command] [-h|--help] [script_file [args...]]\n\n");
            vga::print_str("Description:\n  Launch or execute commands inside the canonical Ring 3 userspace shell.\n\n");
            vga::print_str("Options:\n");
            vga::print_str("  -c <cmd>       Execute command string non-interactively and exit\n");
            vga::print_str("  -h, --help     Show this help message and exit\n");
            vga::print_str("  script_file    Read and execute commands from script file\n");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            return;
        }

        if arg_count < 16 {
            args_buf[arg_count] = part;
            arg_count += 1;
        }
    }

    let sh_path = if keira_fs::exists("/bin/sh.elf") {
        "/bin/sh.elf"
    } else if keira_fs::exists("/bin/sh") {
        "/bin/sh"
    } else {
        vga::set_color(vga::Color::LightRed, vga::Color::Black);
        vga::print_str("sh: /bin/sh.elf: No such file or directory\n");
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        return;
    };

    unsafe {
        if let Err(e) = run_user_program(sh_path, &args_buf[..arg_count]) {
            vga::set_color(vga::Color::LightRed, vga::Color::Black);
            vga::print_str("Error executing userspace shell: ");
            vga::print_str(e);
            vga::print_str("\n");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        }
    }
}
