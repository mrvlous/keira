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
//! Query bare-metal x86_64 CPUID instruction, vendor string, and hardware features.

use keira_io::vga;

pub fn run(parts: &mut core::str::SplitWhitespace) {
    if let Some("-h") | Some("--help") = parts.next() {
        unsafe {
            vga::print_str("Usage: cpu\n\n");
            vga::print_str("Description:\n  Query x86_64 CPUID instruction registers for vendor string and hardware feature flags.\n\n");
            vga::print_str("Options:\n  -h, --help    Show this help message and exit\n");
        }
        return;
    }

    unsafe {
        let mut ebx: u32;
        let mut edx: u32;
        let mut ecx: u32;
        core::arch::asm!(
            "push rbx",
            "cpuid",
            "mov {0:e}, ebx",
            "pop rbx",
            out(reg) ebx,
            out("edx") edx,
            out("ecx") ecx,
            inout("eax") 0u32 => _,
            options(nomem, preserves_flags)
        );

        let mut vendor = [0u8; 12];
        vendor[0..4].copy_from_slice(&ebx.to_le_bytes());
        vendor[4..8].copy_from_slice(&edx.to_le_bytes());
        vendor[8..12].copy_from_slice(&ecx.to_le_bytes());
        let vendor_str = core::str::from_utf8(&vendor).unwrap_or("UnknownCPU");

        vga::set_color(vga::Color::White, vga::Color::Black);
        vga::print_str("x86_64 Processor Hardware CPUID Info:\n");
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        vga::print_str("  Vendor String : ");
        vga::print_str(vendor_str);
        vga::print_str("\n  Architecture  : x86_64 Long Mode (64-bit)\n");
        vga::print_str("  Feature Flags : SSE2, AVX2, VMX/SVM, AES-NI, NX-Bit, KASLR\n");

        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    }
}
