#![allow(unused_variables, unused_unsafe)]
// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Keira Kernel: Shell Command 'reset'
//!
//! Implementation of the 'reset' shell command.

use crate::io::vga;
use crate::shell::executor::*;

pub fn run(parts: &mut core::str::SplitWhitespace) {
    unsafe {
        if let Some("-h") | Some("--help") = parts.next() {
            vga::print_str("Usage: reset\n\n");
            vga::print_str("Description:\n  Reboot the system using a PS/2 keyboard controller hardware reset.\n\n");
            vga::print_str("Options:\n  -h, --help    Show this help message and exit\n");
            return;
        }

        unsafe {
            if !is_admin_mode() {
                vga::set_color(vga::Color::LightRed, vga::Color::Black);
                vga::print_str("Permission denied: This command requires admin privileges. Use 'please <command>'.\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                return;
            }
        }
        vga::print_str("Rebooting...\n");
        unsafe {
            core::arch::asm!(
                "out dx, al",
                in("dx") 0x64u16,
                in("al") 0xFEu8,
                options(nomem, nostack, preserves_flags)
            );
        }
        unsafe {
            let null_idt: [u8; 6] = [0; 6];
            core::arch::asm!(
                "lidt [{}]",
                "int3",
                in(reg) &null_idt,
                options(nostack)
            );
        }
        loop {
            unsafe {
                core::arch::asm!("hlt");
            }
        }
    }
}
