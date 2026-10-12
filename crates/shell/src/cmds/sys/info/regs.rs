// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Display CPU control registers, processor flags and execution mode.

use keira_io::vga;

/// Executes the 'regs' command to dump CPU control registers and execution flags.
pub fn run(parts: &mut core::str::SplitWhitespace) {
    if let Some("-h") | Some("--help") = parts.next() {
        vga::set_color(vga::Color::White, vga::Color::Black);
        vga::print_str("Usage: regs [-h]\n\n");
        vga::print_str(
            "Description:\n  Display CPU control registers (CR0-CR4) and execution flags.\n\n",
        );
        vga::print_str("Options:\n  -h, --help    Show this help message and exit\n");
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        return;
    }

    let cr0 = unsafe { keira_arch::read_cr0() };
    let cr2 = unsafe { keira_arch::read_cr2() };
    let cr3 = unsafe { keira_arch::read_cr3() };
    let cr4 = unsafe { keira_arch::read_cr4() };
    let rflags = unsafe { keira_arch::read_rflags() };

    vga::set_color(vga::Color::White, vga::Color::Black);
    vga::print_str("CPU Control & Execution Registers:\n");

    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    vga::print_str("  CR0:    ");
    vga::print_hex(cr0 as u64);
    vga::print_str(" (");
    if cr0 & (1 << 0) != 0 {
        vga::print_str("PE ");
    }
    if cr0 & (1 << 1) != 0 {
        vga::print_str("MP ");
    }
    if cr0 & (1 << 2) != 0 {
        vga::print_str("EM ");
    }
    if cr0 & (1 << 3) != 0 {
        vga::print_str("TS ");
    }
    if cr0 & (1 << 4) != 0 {
        vga::print_str("ET ");
    }
    if cr0 & (1 << 5) != 0 {
        vga::print_str("NE ");
    }
    if cr0 & (1 << 16) != 0 {
        vga::print_str("WP ");
    }
    if cr0 & (1 << 18) != 0 {
        vga::print_str("AM ");
    }
    if cr0 & (1 << 29) != 0 {
        vga::print_str("NW ");
    }
    if cr0 & (1 << 30) != 0 {
        vga::print_str("CD ");
    }
    if cr0 & (1 << 31) != 0 {
        vga::print_str("PG");
    }
    vga::print_str(")\n");

    vga::print_str("  CR2:    ");
    vga::print_hex(cr2 as u64);
    vga::print_str(" (Page Fault Linear Address)\n");

    vga::print_str("  CR3:    ");
    vga::print_hex(cr3 as u64);
    vga::print_str(" (Page Directory / PML4 Base Address)\n");

    vga::print_str("  CR4:    ");
    vga::print_hex(cr4 as u64);
    vga::print_str(" (");
    if cr4 & (1 << 0) != 0 {
        vga::print_str("VME ");
    }
    if cr4 & (1 << 1) != 0 {
        vga::print_str("PVI ");
    }
    if cr4 & (1 << 2) != 0 {
        vga::print_str("TSD ");
    }
    if cr4 & (1 << 3) != 0 {
        vga::print_str("DE ");
    }
    if cr4 & (1 << 4) != 0 {
        vga::print_str("PSE ");
    }
    if cr4 & (1 << 5) != 0 {
        vga::print_str("PAE ");
    }
    if cr4 & (1 << 7) != 0 {
        vga::print_str("PGE ");
    }
    if cr4 & (1 << 9) != 0 {
        vga::print_str("OSFXSR ");
    }
    if cr4 & (1 << 10) != 0 {
        vga::print_str("OSXMMEXCPT");
    }
    vga::print_str(")\n");

    #[cfg(target_arch = "x86_64")]
    vga::print_str("  RFLAGS: ");
    #[cfg(target_arch = "x86")]
    vga::print_str("  EFLAGS: ");
    vga::print_hex(rflags as u64);
    vga::print_str(" (");
    if rflags & (1 << 0) != 0 {
        vga::print_str("CF ");
    }
    if rflags & (1 << 2) != 0 {
        vga::print_str("PF ");
    }
    if rflags & (1 << 4) != 0 {
        vga::print_str("AF ");
    }
    if rflags & (1 << 6) != 0 {
        vga::print_str("ZF ");
    }
    if rflags & (1 << 7) != 0 {
        vga::print_str("SF ");
    }
    if rflags & (1 << 8) != 0 {
        vga::print_str("TF ");
    }
    if rflags & (1 << 9) != 0 {
        vga::print_str("IF ");
    }
    if rflags & (1 << 10) != 0 {
        vga::print_str("DF ");
    }
    if rflags & (1 << 11) != 0 {
        vga::print_str("OF ");
    }
    vga::print_str("IOPL=");
    vga::print_u64(((rflags >> 12) & 0x3) as u64);
    vga::print_str(")\n");
}
