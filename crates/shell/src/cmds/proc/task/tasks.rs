// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Implementation of the 'tasks' shell command.

use crate::args::CliArgs;
use keira_io::vga;

pub fn run(parts: &mut core::str::SplitWhitespace) {
    let args = CliArgs::parse(parts);

    if args.has_flag('h', "help") {
        {
            vga::set_color(vga::Color::White, vga::Color::Black);
            vga::print_str("Usage: tasks [-a] [-s]\n\n");
            vga::print_str(
                "Description:\n  List all active processes, execution states, and task IDs.\n\n",
            );
            vga::print_str("Options:\n");
            vga::print_str("  -a, --all      Display all kernel worker tasks and threads\n");
            vga::print_str(
                "  -s, --summary  Display total active process count and scheduler summary\n",
            );
            vga::print_str("  -h, --help     Show this help message and exit\n");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        }
        return;
    }

    unsafe {
        if args.has_flag('s', "summary") {
            let (switches, ticks, active) = keira_task::scheduler_get_stats();
            vga::set_color(vga::Color::White, vga::Color::Black);
            vga::print_str("Process & Scheduler Summary: ");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            vga::print_u64(active as u64);
            vga::print_str(" active tasks | ");
            vga::print_u64(switches);
            vga::print_str(" context switches | ");
            vga::print_u64(ticks);
            vga::print_str(" ticks (Max Slots: 64)\n");
            return;
        }

        keira_task::list_tasks();
    }
}
