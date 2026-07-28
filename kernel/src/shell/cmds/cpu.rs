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
    let cpuid = unsafe { core::arch::x86_64::__cpuid(0) };
    let mut vendor = [0u8; 12];
    vendor[0..4].copy_from_slice(&cpuid.ebx.to_le_bytes());
    vendor[4..8].copy_from_slice(&cpuid.edx.to_le_bytes());
    vendor[8..12].copy_from_slice(&cpuid.ecx.to_le_bytes());
    if let Ok(v_str) = core::str::from_utf8(&vendor) {
        unsafe {
            vga::set_color(vga::Color::LightBlue, vga::Color::Black);
            vga::print_str("CPU Vendor: ");
            vga::set_color(vga::Color::White, vga::Color::Black);
            vga::print_str(v_str);
            vga::print_str("\n");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        }
    }
}
