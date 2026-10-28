// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Implementation of the 'rlimit' and 'limit' shell commands.

use crate::args::CliArgs;
use keira_io::vga;
use keira_task::rlimit::{resource_name, resource_unit, RLIM_INFINITY, RLIM_NLIMITS};
use keira_task::scheduler::{CURRENT_TASK_IDX, TASKS};
use keira_task::types::{MAX_FDS, MAX_TASKS};

fn print_padded_str(s: &str, width: usize) {
    vga::print_str(s);
    let len = s.len();
    for _ in len..width {
        vga::print_str(" ");
    }
}

fn print_limit_val(val: u64, width: usize) {
    if val == RLIM_INFINITY {
        print_padded_str("unlimited", width);
    } else {
        let mut digits = 1;
        let mut temp = val;
        while temp >= 10 {
            digits += 1;
            temp /= 10;
        }
        vga::print_u64(val);
        for _ in digits..width {
            vga::print_str(" ");
        }
    }
}

/// Executes the 'rlimit' / 'limit' supervisor diagnostic command.
pub fn run(parts: &mut core::str::SplitWhitespace) {
    let args = CliArgs::parse(parts);

    if args.has_flag('h', "help") {
        vga::set_color(vga::Color::White, vga::Color::Black);
        vga::print_str("Usage: rlimit [-v] [-h]\n\n");
        vga::print_str(
            "Description:\n  Display process resource limits and sandbox quota enforcement telemetry.\n\n",
        );
        vga::print_str("Options:\n");
        vga::print_str("  -v, --verbose  Display per-task resource utilization and quota limits\n");
        vga::print_str("  -h, --help     Show this help message and exit\n");
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        return;
    }

    let is_verbose = args.has_flag('v', "verbose");

    vga::set_color(vga::Color::LightCyan, vga::Color::Black);
    vga::print_str("Keira Process Resource Limits (RLIMIT) Subsystem\n");
    vga::set_color(vga::Color::LightGrey, vga::Color::Black);

    unsafe {
        let current_idx = CURRENT_TASK_IDX;
        let current_task = match TASKS[current_idx] {
            Some(ref t) => t,
            None => {
                vga::print_str("Error: Current task descriptor unavailable\n");
                return;
            }
        };

        vga::print_str("Task PID: ");
        vga::print_u64(current_task.id as u64);
        vga::print_str(" (");
        vga::print_str(current_task.name);
        vga::print_str(")\n\n");

        vga::set_color(vga::Color::Yellow, vga::Color::Black);
        print_padded_str("Limit", 26);
        print_padded_str("Soft Limit", 20);
        print_padded_str("Hard Limit", 20);
        print_padded_str("Units", 12);
        vga::print_str("\n");

        vga::set_color(vga::Color::DarkGrey, vga::Color::Black);
        vga::print_str(
            "-------------------------------------------------------------------------------\n",
        );
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);

        for res in 0..RLIM_NLIMITS {
            let rlim = current_task.rlimits[res];
            let name = resource_name(res);
            let unit = resource_unit(res);

            print_padded_str(name, 26);
            print_limit_val(rlim.cur, 20);
            print_limit_val(rlim.max, 20);
            print_padded_str(unit, 12);
            vga::print_str("\n");
        }

        if is_verbose {
            vga::print_str("\n");
            vga::set_color(vga::Color::LightCyan, vga::Color::Black);
            vga::print_str("Active Task Sandboxing & Quota Utilization:\n");
            vga::set_color(vga::Color::Yellow, vga::Color::Black);
            print_padded_str("PID", 6);
            print_padded_str("Name", 16);
            print_padded_str("Open FDs", 12);
            print_padded_str("FD Limit", 12);
            print_padded_str("Heap (KiB)", 14);
            print_padded_str("Data Limit", 14);
            vga::print_str("\n");

            vga::set_color(vga::Color::DarkGrey, vga::Color::Black);
            vga::print_str(
                "------------------------------------------------------------------------------\n",
            );
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);

            for i in 0..MAX_TASKS {
                if let Some(ref t) = TASKS[i] {
                    let mut open_fds = 0;
                    for fd in &t.fds {
                        if fd.is_open {
                            open_fds += 1;
                        }
                    }
                    let heap_kb = t.program_break.saturating_sub(t.program_break_start) / 1024;
                    let fd_limit = t.rlimits[keira_task::rlimit::RLIMIT_NOFILE]
                        .cur
                        .min(MAX_FDS as u64);

                    let mut digits = 1;
                    let mut temp = t.id as u64;
                    while temp >= 10 {
                        digits += 1;
                        temp /= 10;
                    }
                    vga::print_u64(t.id as u64);
                    for _ in digits..6 {
                        vga::print_str(" ");
                    }

                    print_padded_str(t.name, 16);

                    let mut ofd_digits = 1;
                    let mut ofd_temp = open_fds;
                    while ofd_temp >= 10 {
                        ofd_digits += 1;
                        ofd_temp /= 10;
                    }
                    vga::print_u64(open_fds as u64);
                    for _ in ofd_digits..12 {
                        vga::print_str(" ");
                    }

                    print_limit_val(fd_limit, 12);

                    let mut hk_digits = 1;
                    let mut hk_temp = heap_kb;
                    while hk_temp >= 10 {
                        hk_digits += 1;
                        hk_temp /= 10;
                    }
                    vga::print_u64(heap_kb);
                    for _ in hk_digits..14 {
                        vga::print_str(" ");
                    }

                    print_limit_val(t.rlimits[keira_task::rlimit::RLIMIT_DATA].cur, 14);
                    vga::print_str("\n");
                }
            }
        }
    }
}
