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
//! Implementation of the 'fg' shell command to bring background process job to terminal foreground.

use keira_io::vga;
use keira_task::signal;

pub fn run(parts: &mut core::str::SplitWhitespace) {
    let arg = parts.next();
    if arg == Some("-h") || arg == Some("--help") {
        unsafe {
            vga::print_str("Usage: fg [job_id]\n\n");
            vga::print_str("Description:\n  Bring background or stopped process job to terminal foreground context.\n\n");
            vga::print_str("Options:\n  -h, --help    Show this help message and exit\n");
        }
        return;
    }

    let job_id = arg.and_then(|s| s.parse::<u32>().ok()).unwrap_or(1);

    unsafe {
        let mut found = false;
        for i in 0..signal::JOB_COUNT {
            if let Some(ref mut job) = signal::JOB_TABLE[i] {
                if job.job_id == job_id {
                    job.is_foreground = true;
                    job.state = signal::JobState::Running;
                    found = true;

                    vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                    vga::print_str("Brought Job [");
                    vga::print_u64(job.job_id as u64);
                    vga::print_str("] (PID ");
                    vga::print_u64(job.pid as u64);
                    vga::print_str(") to terminal foreground context.\n");
                    break;
                }
            }
        }

        if !found {
            vga::set_color(vga::Color::Yellow, vga::Color::Black);
            vga::print_str("fg: Job [");
            vga::print_u64(job_id as u64);
            vga::print_str("] not found in Job Control Table.\n");
        }
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    }
}
