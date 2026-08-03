#![allow(unused_variables, unused_unsafe)]
// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Keira Kernel: Shell Command 'cpu'
//!
//! Implementation of the 'cpu' shell command to display CPU vendor signatures.

use crate::io::vga;

pub fn run(parts: &mut core::str::SplitWhitespace) {
    if let Some("-h") | Some("--help") = parts.next() {
        unsafe {
            vga::print_str("Usage: cpu\n\n");
            vga::print_str("Description:\n  Query CPUID instruction to display processor vendor string, 64-bit architecture, APIC controller state, and SMP active core counts.\n\n");
            vga::print_str("Options:\n  -h, --help    Show this help message and exit\n");
        }
        return;
    }

    let cpuid = unsafe { core::arch::x86_64::__cpuid(0) };
    let mut vendor = [0u8; 12];
    vendor[0..4].copy_from_slice(&cpuid.ebx.to_le_bytes());
    vendor[4..8].copy_from_slice(&cpuid.edx.to_le_bytes());
    vendor[8..12].copy_from_slice(&cpuid.ecx.to_le_bytes());
    if let Ok(v_str) = core::str::from_utf8(&vendor) {
        unsafe {
            vga::set_color(vga::Color::LightBlue, vga::Color::Black);
            vga::print_str("CPU VENDOR: ");
            vga::set_color(vga::Color::White, vga::Color::Black);
            vga::print_str(v_str);
            vga::print_str("\n");

            vga::set_color(vga::Color::LightCyan, vga::Color::Black);
            vga::print_str("  Architecture : x86_64 64-Bit Long Mode\n");
            vga::print_str("  APIC Controller : ");
            if crate::arch::apic::APIC_INITIALIZED {
                vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                vga::print_str("Enabled (MMIO 0xFEE00000)\n");
            } else {
                vga::set_color(vga::Color::Yellow, vga::Color::Black);
                vga::print_str("PIC Fallback Mode\n");
            }

            vga::set_color(vga::Color::LightCyan, vga::Color::Black);
            vga::print_str("  SMP Cores    : ");
            vga::set_color(vga::Color::White, vga::Color::Black);
            vga::print_u64(crate::arch::apic::CPU_CORE_COUNT as u64);
            vga::print_str(" Active Core(s)\n");

            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        }
    }
}
