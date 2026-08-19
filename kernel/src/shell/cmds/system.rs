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
//! Implementation of the 'system' shell command to display kernel specifications,
//! CPU architecture information, memory utilization, and system uptime.

use crate::io::vga;
use crate::shell::executor::*;

pub fn run(parts: &mut core::str::SplitWhitespace) {
    if let Some("-h") | Some("--help") = parts.next() {
        unsafe {
            vga::print_str("Usage: system\n\n");
            vga::print_str("Description:\n  Display comprehensive kernel specifications, OS version, CPU vendor, system uptime, heap utilization, and PCI device counts.\n\n");
            vga::print_str("Options:\n  -h, --help    Show this help message and exit\n");
        }
        return;
    }

    unsafe {
        let ms = get_uptime_ms();
        let hours = ms / 3600000;
        let minutes = (ms % 3600000) / 60000;
        let seconds = (ms % 60000) / 1000;
        let millis = ms % 1000;

        let cpuid = core::arch::x86_64::__cpuid(0);
        let mut vendor = [0u8; 12];
        vendor[0..4].copy_from_slice(&cpuid.ebx.to_le_bytes());
        vendor[4..8].copy_from_slice(&cpuid.edx.to_le_bytes());
        vendor[8..12].copy_from_slice(&cpuid.ecx.to_le_bytes());

        let pci_count = count_pci_devices();
        let heap_total = heap_get_total() as u64;
        let heap_used = heap_get_used() as u64;

        vga::set_color(vga::Color::LightCyan, vga::Color::Black);
        vga::print_str("System Specifications & Kernel Information\n");

        vga::set_color(vga::Color::LightBlue, vga::Color::Black);
        vga::print_str("  OS System Version : ");
        vga::set_color(vga::Color::White, vga::Color::Black);
        vga::print_str("Keira Kernel v");
        vga::print_str(env!("CARGO_PKG_VERSION"));
        vga::print_str("\n");

        vga::set_color(vga::Color::LightBlue, vga::Color::Black);
        vga::print_str("  Architecture      : ");
        vga::set_color(vga::Color::White, vga::Color::Black);
        vga::print_str("x86_64 Long Mode (Freestanding)\n");

        vga::set_color(vga::Color::LightBlue, vga::Color::Black);
        vga::print_str("  CPU Vendor        : ");
        vga::set_color(vga::Color::White, vga::Color::Black);
        if let Ok(v_str) = core::str::from_utf8(&vendor) {
            vga::print_str(v_str);
        }
        vga::print_str("\n");

        vga::set_color(vga::Color::LightBlue, vga::Color::Black);
        vga::print_str("  System Uptime     : ");
        vga::set_color(vga::Color::White, vga::Color::Black);
        vga::print_u64(hours);
        vga::print_str("h ");
        vga::print_u64(minutes);
        vga::print_str("m ");
        vga::print_u64(seconds);
        vga::print_str("s ");
        vga::print_u64(millis);
        vga::print_str("ms\n");

        vga::set_color(vga::Color::LightBlue, vga::Color::Black);
        vga::print_str("  Heap Memory       : ");
        vga::set_color(vga::Color::White, vga::Color::Black);
        vga::print_u64(heap_used / 1024);
        vga::print_str(" KB / ");
        vga::print_u64(heap_total / 1024);
        vga::print_str(" KB\n");

        vga::set_color(vga::Color::LightBlue, vga::Color::Black);
        vga::print_str("  PCI Devices       : ");
        vga::set_color(vga::Color::White, vga::Color::Black);
        vga::print_u64(pci_count);
        vga::print_str(" detected\n");

        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    }
}
