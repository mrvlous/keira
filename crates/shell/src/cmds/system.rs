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

use crate::args::CliArgs;
use crate::executor::*;
use keira_io::vga;

pub fn run(parts: &mut core::str::SplitWhitespace) {
    let args = CliArgs::parse(parts);

    if args.has_flag('h', "help") {
        unsafe {
            vga::set_color(vga::Color::White, vga::Color::Black);
            vga::print_str("Usage: system [-v] [-u] [-s]\n\n");
            vga::print_str("Description:\n  Display kernel specifications, CPU architecture, and system telemetry.\n\n");
            vga::print_str("Options:\n");
            vga::print_str("  -v, --version  Display kernel version, build target, and license\n");
            vga::print_str("  -u, --uptime   Display system uptime duration\n");
            vga::print_str("  -s, --summary  Display compact one-line system status\n");
            vga::print_str("  -h, --help     Show this help message and exit\n");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        }
        return;
    }

    unsafe {
        let ms = get_uptime_ms();
        let hours = ms / 3600000;
        let minutes = (ms % 3600000) / 60000;
        let seconds = (ms % 60000) / 1000;
        let millis = ms % 1000;

        if args.has_flag('u', "uptime") {
            vga::set_color(vga::Color::White, vga::Color::Black);
            vga::print_str("Uptime: ");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            vga::print_u64(hours);
            vga::print_str("h ");
            vga::print_u64(minutes);
            vga::print_str("m ");
            vga::print_u64(seconds);
            vga::print_str("s ");
            vga::print_u64(millis);
            vga::print_str("ms\n");
            return;
        }

        if args.has_flag('v', "version") {
            vga::set_color(vga::Color::White, vga::Color::Black);
            vga::print_str("Keira Kernel 0.33.7-keira-1 (x86_64-unknown-none)\n");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            vga::print_str("Compiled with rustc 1.85+ (LLVM 19), GPL-2.0-only\n");
            vga::print_str("Author: Moh. Ananda Firmansyah Putra\n");
            return;
        }

        if args.has_flag('s', "summary") {
            vga::set_color(vga::Color::White, vga::Color::Black);
            vga::print_str("Keira v0.33.7 | x86_64 | Up: ");
            vga::print_u64(hours);
            vga::print_str("h ");
            vga::print_u64(minutes);
            vga::print_str("m ");
            vga::print_u64(seconds);
            vga::print_str("s\n");
            return;
        }

        let cpuid = core::arch::x86_64::__cpuid(0);
        let mut vendor = [0u8; 12];
        vendor[0..4].copy_from_slice(&cpuid.ebx.to_le_bytes());
        vendor[4..8].copy_from_slice(&cpuid.edx.to_le_bytes());
        vendor[8..12].copy_from_slice(&cpuid.ecx.to_le_bytes());

        let pci_count = count_pci_devices();
        let heap_total = heap_get_total() as u64;
        let heap_used = heap_get_used() as u64;

        vga::set_color(vga::Color::White, vga::Color::Black);
        vga::print_str("System Specifications & Kernel Information\n");

        vga::print_str("  OS System Version : ");
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        vga::print_str("Keira Kernel v");
        vga::print_str(env!("CARGO_PKG_VERSION"));
        vga::print_str("\n");

        vga::set_color(vga::Color::White, vga::Color::Black);
        vga::print_str("  Architecture      : ");
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        vga::print_str("x86_64 Long Mode (Freestanding)\n");

        vga::set_color(vga::Color::White, vga::Color::Black);
        vga::print_str("  CPU Vendor        : ");
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        if let Ok(v) = core::str::from_utf8(&vendor) {
            vga::print_str(v);
        }
        vga::print_str("\n");

        vga::set_color(vga::Color::White, vga::Color::Black);
        vga::print_str("  System Uptime     : ");
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        vga::print_u64(hours);
        vga::print_str("h ");
        vga::print_u64(minutes);
        vga::print_str("m ");
        vga::print_u64(seconds);
        vga::print_str("s ");
        vga::print_u64(millis);
        vga::print_str("ms\n");

        vga::set_color(vga::Color::White, vga::Color::Black);
        vga::print_str("  Heap Memory       : ");
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        vga::print_u64(heap_used / 1024);
        vga::print_str(" KB / ");
        vga::print_u64(heap_total / 1024);
        vga::print_str(" KB\n");

        vga::set_color(vga::Color::White, vga::Color::Black);
        vga::print_str("  PCI Devices       : ");
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        vga::print_u64(pci_count as u64);
        vga::print_str(" detected\n");
    }
}
