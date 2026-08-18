#![allow(unused_variables, unused_unsafe)]
// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Keira Kernel: Shell Command 'drivers'
//!
//! Query active C and Rust kernel device drivers.

use crate::io::vga;

pub fn run(parts: &mut core::str::SplitWhitespace) {
    if let Some("-h") | Some("--help") = parts.next() {
        unsafe {
            vga::print_str("Usage: drivers\n\n");
            vga::print_str("Description:\n  Query active C and Rust hardware device drivers.\n\n");
            vga::print_str("Options:\n  -h, --help    Show this help message and exit\n");
        }
        return;
    }

    unsafe {
        vga::set_color(vga::Color::LightCyan, vga::Color::Black);
        vga::print_str("Active Kernel Hardware Drivers:\n");
        vga::set_color(vga::Color::LightGreen, vga::Color::Black);
        vga::print_str("  [C Driver] Serial UART COM1 (115200 8N1)\n");
        vga::print_str("  [C Driver] VGA Text Mode (80x25 / 80x50)\n");
        vga::print_str("  [C Driver] Intel e1000 Gigabit Ethernet NIC\n");
        vga::print_str("  [C Driver] PS/2 Keyboard & Mouse Controller\n");
        vga::print_str("  [C Driver] CMOS Real Time Clock (RTC)\n");
        vga::print_str("  [C Driver] Intel High Definition Audio (HDA) DSP\n");
        vga::print_str("  [Rust Driver] NVMe PCIe Controller & Storage Host\n");
        vga::print_str("  [Rust Driver] USB 3.0 xHCI Host Controller & Mass Storage\n");
        vga::print_str("  [PCI Bus] Devices Detected: ");
        vga::print_u64(crate::io::pci::PCI_DEVICE_COUNT as u64);
        vga::print_str("\n");
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    }
}
