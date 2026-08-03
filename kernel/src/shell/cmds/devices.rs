#![allow(unused_variables, unused_unsafe)]
// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Keira Kernel: Shell Command 'devices'
//!
//! Implementation of the 'devices' shell command to scan and list PCI devices.

use crate::io::vga;
use crate::shell::executor::*;

fn print_hex_u16(val: u16) {
    let chars = b"0123456789ABCDEF";
    let mut buf = [0u8; 4];
    buf[0] = chars[((val >> 12) & 0xF) as usize];
    buf[1] = chars[((val >> 8) & 0xF) as usize];
    buf[2] = chars[((val >> 4) & 0xF) as usize];
    buf[3] = chars[(val & 0xF) as usize];
    if let Ok(s) = core::str::from_utf8(&buf) {
        vga::print_str(s);
    }
}

pub fn run(parts: &mut core::str::SplitWhitespace) {
    unsafe {
        if let Some("-h") | Some("--help") = parts.next() {
            vga::print_str("Usage: devices\n\n");
            vga::print_str("Description:\n  Scan PCI bus slots and enumerate detected hardware devices, vendor IDs, device IDs, and class types.\n\n");
            vga::print_str("Options:\n  -h, --help    Show this help message and exit\n");
            return;
        }

        if !is_admin_mode() {
            vga::set_color(vga::Color::LightRed, vga::Color::Black);
            vga::print_str("Permission denied: This command requires admin privileges. Use 'please <command>'.\n");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            return;
        }

        // Initialize/rescan the PCI bus
        crate::io::pci::init();

        vga::set_color(vga::Color::LightBlue, vga::Color::Black);
        vga::print_str("BUS   SLOT  FUNC  VENDOR  DEVICE  CLASS TYPE\n");
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);

        for i in 0..crate::io::pci::PCI_DEVICE_COUNT {
            if let Some(dev) = crate::io::pci::PCI_DEVICES[i] {
                // Print bus
                vga::print_u64(dev.bus as u64);
                vga::print_str("     ");

                // Print slot
                vga::print_u64(dev.slot as u64);
                if dev.slot < 10 {
                    vga::print_str("     ");
                } else {
                    vga::print_str("    ");
                }

                // Print func
                vga::print_u64(dev.func as u64);
                vga::print_str("     ");

                // Print Vendor ID
                print_hex_u16(dev.vendor_id);
                vga::print_str("    ");

                // Print Device ID
                print_hex_u16(dev.device_id);
                vga::print_str("    ");

                // Print Class Description
                let class_str = crate::io::pci::pci_class_to_str(dev.class_code, dev.subclass);
                vga::print_str(class_str);
                vga::print_str("\n");
            }
        }
    }
}
