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
//! Implementation of the 'tasks' shell command.

use crate::args::CliArgs;
use crate::executor::*;
use keira_io::vga;

pub fn run(parts: &mut core::str::SplitWhitespace) {
    let args = CliArgs::parse(parts);

    if args.has_flag('h', "help") {
        unsafe {
            vga::set_color(vga::Color::White, vga::Color::Black);
            vga::print_str("Usage: tasks [-a] [-s]\n\n");
            vga::print_str(
                "Description:\n  List all active processes, execution states, and task IDs.\n\n",
            );
            vga::print_str("Options:\n");
            vga::print_str("  -a, --all      Display all kernel worker tasks and threads\n");
            vga::print_str("  -s, --summary  Display total active process count summary\n");
            vga::print_str("  -h, --help     Show this help message and exit\n");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        }
        return;
    }

    unsafe {
        if !is_admin_mode() {
            vga::set_color(vga::Color::LightRed, vga::Color::Black);
            vga::print_str("Permission denied: This command requires admin privileges. Use 'please <command>'.\n");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            return;
        }

        if args.has_flag('s', "summary") {
            vga::set_color(vga::Color::White, vga::Color::Black);
            vga::print_str("Process Summary: ");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            vga::print_str("1 running, 0 sleeping, Max Slots: 16\n");
            return;
        }

        keira_task::list_tasks();
    }
}
