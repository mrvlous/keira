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
//! Implementation of the 'jobs' shell command to list active background and stopped process jobs.

use keira_io::vga;
use keira_task::signal;

pub fn run(parts: &mut core::str::SplitWhitespace) {
    let arg = parts.next();
    if arg == Some("-h") || arg == Some("--help") {
        unsafe {
            vga::print_str("Usage: jobs\n\n");
            vga::print_str("Description:\n  List active background and stopped process jobs in Job Control Table.\n\n");
            vga::print_str("Options:\n  -h, --help    Show this help message and exit\n");
        }
        return;
    }

    unsafe {
        vga::set_color(vga::Color::White, vga::Color::Black);
        vga::print_str("  [Job ID]  PID    State       Command\n");
        vga::print_str("  --------  -----  ----------  -------------------------\n");

        let mut count = 0;
        for i in 0..signal::JOB_COUNT {
            if let Some(ref job) = signal::JOB_TABLE[i] {
                count += 1;
                vga::print_str("  [");
                vga::print_u64(job.job_id as u64);
                vga::print_str("]       ");
                vga::print_u64(job.pid as u64);
                vga::print_str("    ");
                match job.state {
                    signal::JobState::Running => vga::print_str("Running     "),
                    signal::JobState::Stopped => vga::print_str("Stopped     "),
                    signal::JobState::Terminated => vga::print_str("Terminated  "),
                }
                if let Ok(name) = core::str::from_utf8(&job.name[..job.name_len]) {
                    vga::print_str(name);
                }
                vga::print_str("\n");
            }
        }

        if count == 0 {
            vga::print_str("  (No active background process jobs currently running)\n");
        }
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    }
}
