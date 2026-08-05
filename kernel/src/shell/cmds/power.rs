#![allow(unused_variables, unused_unsafe)]
// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Keira Kernel: Shell Command 'power' / 'poweroff' / 'reboot'
//!
//! Manage ACPI power state transitions and NMI hardware watchdog timer.

use crate::io::vga;

pub fn run(parts: &mut core::str::SplitWhitespace) {
    if let Some("-h") | Some("--help") = parts.next() {
        unsafe {
            vga::print_str("Usage: power [status|off|sleep]\n\n");
            vga::print_str("Description:\n  Manage ACPI power state transitions (S0/S3/S5) & NMI hardware watchdog timer.\n\n");
            vga::print_str("Options:\n  -h, --help    Show this help message and exit\n");
        }
        return;
    }

    unsafe {
        vga::set_color(vga::Color::LightCyan, vga::Color::Black);
        vga::print_str("ACPI Power Management & NMI Watchdog Status\n");
        let _ = crate::arch::power::set_power_state(0);
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    }
}
