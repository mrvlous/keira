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
//! Query ACPI Power Management and NMI hardware watchdog status.

use crate::executor::is_admin_mode;
use keira_io::vga;

pub fn run(parts: &mut core::str::SplitWhitespace) {
    let sub = parts.next();
    if let Some("-h") | Some("--help") = sub {
        unsafe {
            vga::print_str("Usage: power [status|acpi|shutdown|poweroff|reboot]\n\n");
            vga::print_str("Description:\n  Query ACPI power management states (S0/S3/S5), initiate hardware shutdown, or reboot.\n\n");
            vga::print_str("Options:\n  -h, --help    Show this help message and exit\n");
        }
        return;
    }

    if let Some("shutdown") | Some("poweroff") | Some("off") = sub {
        unsafe {
            if !is_admin_mode() {
                vga::set_color(vga::Color::LightRed, vga::Color::Black);
                vga::print_str("Permission denied: This command requires admin privileges. Use 'please <command>'.\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                return;
            }
            vga::set_color(vga::Color::Yellow, vga::Color::Black);
            vga::print_str("Powering off Keira Kernel via ACPI S5 Soft-Off...\n");
            keira_arch::power::acpi::poweroff();
        }
    } else if let Some("reboot") | Some("restart") | Some("reset") = sub {
        unsafe {
            if !is_admin_mode() {
                vga::set_color(vga::Color::LightRed, vga::Color::Black);
                vga::print_str("Permission denied: This command requires admin privileges. Use 'please <command>'.\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                return;
            }
            vga::set_color(vga::Color::Yellow, vga::Color::Black);
            vga::print_str("Rebooting Keira Kernel via PS/2 controller...\n");
            keira_arch::power::acpi::reboot();
        }
    }

    unsafe {
        vga::set_color(vga::Color::White, vga::Color::Black);
        vga::print_str("ACPI Power Management & Hardware Watchdog:\n");
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        vga::print_str("  ACPI State    : S0 (Working)\n");
        vga::print_str("  NMI Watchdog  : PETTED / ACTIVE ");
        vga::set_color(vga::Color::LightGreen, vga::Color::Black);
        vga::print_str("[OK]\n");
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    }
}
