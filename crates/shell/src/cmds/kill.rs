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
//! Implementation of the 'kill' shell command to dispatch POSIX signals to processes (Syscall 72).

use keira_io::vga;
use keira_task::signal;

pub fn run(parts: &mut core::str::SplitWhitespace) {
    let arg1 = parts.next();
    if arg1 == Some("-h") || arg1 == Some("--help") {
        unsafe {
            vga::print_str("Usage: kill [-signal_number] <pid>\n\n");
            vga::print_str(
                "Description:\n  Dispatch POSIX real-time signal to process PID (Syscall 72).\n\n",
            );
            vga::print_str("Options:\n  -9, -SIGKILL    Send SIGKILL (Force kill)\n  -15, -SIGTERM   Send SIGTERM (Graceful termination)\n  -2, -SIGINT     Send SIGINT (Terminal interrupt)\n");
        }
        return;
    }

    let mut sig = signal::SIGTERM;
    let mut pid_str = arg1;

    if let Some(first) = arg1 {
        if let Some(sig_name) = first.strip_prefix('-') {
            sig = match sig_name {
                "9" | "SIGKILL" | "KILL" => signal::SIGKILL,
                "2" | "SIGINT" | "INT" => signal::SIGINT,
                "15" | "SIGTERM" | "TERM" => signal::SIGTERM,
                "19" | "SIGSTOP" | "STOP" => signal::SIGSTOP,
                "18" | "SIGCONT" | "CONT" => signal::SIGCONT,
                _ => signal::SIGTERM,
            };
            pid_str = parts.next();
        }
    }

    if let Some(ps) = pid_str {
        if let Ok(pid) = ps.parse::<u32>() {
            let _ = signal::sys_kill(pid, sig);
        } else {
            unsafe {
                vga::set_color(vga::Color::LightRed, vga::Color::Black);
                vga::print_str("kill: Invalid process PID\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            }
        }
    } else {
        unsafe {
            vga::set_color(vga::Color::Yellow, vga::Color::Black);
            vga::print_str("Usage: kill [-signal_number] <pid>\n");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        }
    }
}
