#![allow(unused_variables, unused_unsafe)]
// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Keira Kernel: Shell Command 'stop'
//!
//! Implementation of the 'stop' shell command to terminate running tasks.

use crate::io::vga;
use crate::shell::executor::*;

pub fn run(parts: &mut core::str::SplitWhitespace) {
    unsafe {
        if !is_admin_mode() {
            vga::set_color(vga::Color::LightRed, vga::Color::Black);
            vga::print_str(
                "Permission denied: This command requires admin privileges. Use 'please <command>'.\n",
            );
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            return;
        }

        let pid_str = match parts.next() {
            Some("-h") | Some("--help") => {
                vga::print_str("Usage: stop <PID>\n\n");
                vga::print_str("Description:\n  Terminate a running process task by its numeric Process ID (PID).\n\n");
                vga::print_str("Options:\n  -h, --help    Show this help message and exit\n\n");
                vga::print_str("Examples:\n  stop 2\n");
                return;
            }
            Some(s) => s,
            None => {
                vga::set_color(vga::Color::Yellow, vga::Color::Black);
                vga::print_str("Usage: stop <PID>\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                return;
            }
        };

        let mut pid: usize = 0;
        for byte in pid_str.bytes() {
            if (b'0'..=b'9').contains(&byte) {
                pid = pid * 10 + (byte - b'0') as usize;
            } else {
                vga::set_color(vga::Color::LightRed, vga::Color::Black);
                vga::print_str("Error: Invalid PID format. Please specify a numeric PID.\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                return;
            }
        }

        match crate::task::scheduler::stop_task(pid) {
            Ok(()) => {
                vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                vga::print_str("Task [PID ");
                vga::print_u64(pid as u64);
                vga::print_str("] successfully terminated.\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            }
            Err(err) => {
                vga::set_color(vga::Color::LightRed, vga::Color::Black);
                vga::print_str("Error: ");
                vga::print_str(err);
                vga::print_str("\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            }
        }
    }
}
