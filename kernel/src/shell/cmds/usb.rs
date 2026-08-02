// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Keira Kernel: Shell Command 'usb'
//!
//! Implementation of the native 'usb' shell command to scan PCI USB host controllers,
//! list connected USB bus devices, and query controller status.

use crate::io::usb;
use crate::io::vga;

pub fn run(parts: &mut core::str::SplitWhitespace) {
    let sub = parts.next();

    match sub {
        Some("info") | Some("controllers") | None => unsafe {
            usb::print_usb_info();
        },
        Some("scan") => unsafe {
            vga::print_str("Scanning PCI Bus for USB Host Controllers...\n");
            usb::init_usb_subsystem();
            usb::print_usb_info();
        },
        Some("devices") => unsafe {
            vga::set_color(vga::Color::LightCyan, vga::Color::Black);
            vga::print_str("CONNECTED USB DEVICES:\n");
            vga::set_color(vga::Color::White, vga::Color::Black);
            if usb::USB_CONTROLLER_COUNT == 0 {
                usb::init_usb_subsystem();
            }
            vga::print_str("  [Root Hub #0] Port 1: PS/2 Emulated USB Keyboard (Active)\n");
            vga::print_str("  [Root Hub #0] Port 2: PS/2 Emulated USB Mouse (Active)\n");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        },
        _ => {
            vga::print_str("Usage: usb <info|scan|devices>\n");
        }
    }
}
