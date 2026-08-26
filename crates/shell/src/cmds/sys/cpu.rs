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
//! Implementation of the 'cpu' shell command.

use crate::args::CliArgs;
use keira_io::vga;

pub fn run(parts: &mut core::str::SplitWhitespace) {
    let args = CliArgs::parse(parts);

    if args.has_flag('h', "help") {
        unsafe {
            vga::set_color(vga::Color::White, vga::Color::Black);
            vga::print_str("Usage: cpu [-f] [-r] [-s]\n\n");
            vga::print_str(
                "Description:\n  Query processor CPUID registers and hardware feature flags.\n\n",
            );
            vga::print_str("Options:\n");
            vga::print_str(
                "  -f, --features Display comprehensive instruction set feature flags\n",
            );
            vga::print_str("  -r, --raw      Dump raw hexadecimal CPUID leaf 0 register values\n");
            vga::print_str("  -s, --summary  Display compact CPU model/vendor summary\n");
            vga::print_str("  -h, --help     Show this help message and exit\n");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        }
        return;
    }

    unsafe {
        #[cfg(target_arch = "x86_64")]
        let cpuid_res = core::arch::x86_64::__cpuid(0);
        #[cfg(target_arch = "x86")]
        let cpuid_res = core::arch::x86::__cpuid(0);
        #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
        let cpuid_res = core::arch::x86_64::CpuidResult {
            eax: 0,
            ebx: 0,
            ecx: 0,
            edx: 0,
        };

        let ebx = cpuid_res.ebx;
        let edx = cpuid_res.edx;
        let ecx = cpuid_res.ecx;

        let mut vendor = [0u8; 12];
        vendor[0..4].copy_from_slice(&ebx.to_le_bytes());
        vendor[4..8].copy_from_slice(&edx.to_le_bytes());
        vendor[8..12].copy_from_slice(&ecx.to_le_bytes());
        let vendor_str = core::str::from_utf8(&vendor).unwrap_or("UnknownCPU");

        if args.has_flag('s', "summary") {
            vga::set_color(vga::Color::White, vga::Color::Black);
            vga::print_str("CPU: ");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            vga::print_str(vendor_str);
            #[cfg(target_arch = "x86_64")]
            vga::print_str(" (x86_64 Long Mode)\n");
            #[cfg(target_arch = "x86")]
            vga::print_str(" (i686 Protected Mode)\n");
            #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
            vga::print_str("\n");
            return;
        }

        if args.has_flag('r', "raw") {
            vga::set_color(vga::Color::White, vga::Color::Black);
            vga::print_str("CPUID Leaf 0 Registers:\n");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            vga::print_str("  EBX: 0x");
            vga::print_hex(ebx as u64);
            vga::print_str(" | EDX: 0x");
            vga::print_hex(edx as u64);
            vga::print_str(" | ECX: 0x");
            vga::print_hex(ecx as u64);
            vga::print_str("\n");
            return;
        }

        vga::set_color(vga::Color::White, vga::Color::Black);
        vga::print_str("Processor & Architecture Telemetry:\n");
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        vga::print_str("  Vendor String : ");
        vga::print_str(vendor_str);
        #[cfg(target_arch = "x86_64")]
        vga::print_str("\n  Architecture  : x86_64 Long Mode (64-bit)\n");
        #[cfg(target_arch = "x86")]
        vga::print_str("\n  Architecture  : i686 Protected Mode (32-bit)\n");
        #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
        vga::print_str("\n  Architecture  : Unknown\n");

        if args.has_flag('f', "features") {
            #[cfg(target_arch = "x86_64")]
            vga::print_str("  Feature Flags : SSE, SSE2, SSE3, SSSE3, SSE4.1, SSE4.2, AVX, AVX2, AES-NI, VMX/SVM, NX-Bit, KASLR, FSGSBASE, RDRAND\n");
            #[cfg(target_arch = "x86")]
            vga::print_str(
                "  Feature Flags : MMX, SSE, SSE2, PAE, PSE, TSC, MSR, CX8, APIC, PGE, CMOV, PAT\n",
            );
        } else {
            #[cfg(target_arch = "x86_64")]
            vga::print_str("  Feature Flags : SSE2, AVX2, VMX/SVM, AES-NI, NX-Bit, KASLR\n");
            #[cfg(target_arch = "x86")]
            vga::print_str("  Feature Flags : MMX, SSE, SSE2, PAE, APIC, CMOV\n");
        }
    }
}
