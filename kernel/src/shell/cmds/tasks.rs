#![allow(unused_variables, unused_unsafe)]
// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Keira Kernel: Shell Command 'tasks'
//!
//! Implementation of the 'tasks' shell command.

use crate::io::vga;
use crate::shell::executor::*;

pub fn run(parts: &mut core::str::SplitWhitespace) {
    unsafe {
        if let Some("-h") | Some("--help") = parts.next() {
            vga::print_str("Usage: tasks\n\n");
            vga::print_str("Description:\n  List all active task processes, Process IDs (PIDs), execution states, and priority queue levels.\n\n");
            vga::print_str("Options:\n  -h, --help    Show this help message and exit\n");
            return;
        }

        if !is_admin_mode() {
            vga::set_color(vga::Color::LightRed, vga::Color::Black);
            vga::print_str("Permission denied: This command requires admin privileges. Use 'please <command>'.\n");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        } else {
            crate::task::list_tasks();
        }
    }
}
