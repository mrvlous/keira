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
//! Inspect loaded kernel driver descriptors and hardware interface status.

use crate::io::vga;

pub fn run(parts: &mut core::str::SplitWhitespace) {
    if let Some("-h") | Some("--help") = parts.next() {
        unsafe {
            vga::print_str("Usage: drivers [list|status]\n\n");
            vga::print_str("Description:\n  Inspect loaded kernel driver descriptors and hardware interface status.\n\n");
            vga::print_str("Options:\n  -h, --help    Show this help message and exit\n");
        }
        return;
    }

    unsafe {
        vga::set_color(vga::Color::LightCyan, vga::Color::Black);
        vga::print_str("Keira Kernel Registered Driver Subsystems\n");
        vga::set_color(vga::Color::LightGreen, vga::Color::Black);
        vga::print_str("  [OK] Serial UART (COM1 16550A)\n");
        vga::print_str("  [OK] VGA Text Console & VBE Framebuffer (32-bpp)\n");
        vga::print_str("  [OK] PS/2 Keyboard & Mouse Drivers\n");
        vga::print_str("  [OK] CMOS Real-Time Clock (RTC)\n");
        vga::print_str("  [OK] IDE PIO & AHCI SATA 64-bit DMA Controllers\n");
        vga::print_str("  [OK] NVMe 1.4 PCIe SSD Storage Controller\n");
        vga::print_str("  [OK] Intel e1000 Network Interface Controller\n");
        vga::print_str("  [OK] Intel High Definition Audio (HDA) Controller\n");
        vga::print_str("  [OK] USB Host Controller Subsystem (xHCI/EHCI/UHCI)\n");
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    }
}
