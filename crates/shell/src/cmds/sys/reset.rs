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
//! Implementation of the 'reset' shell command.

use crate::executor::*;
use keira_io::vga;

pub fn run(_parts: &mut core::str::SplitWhitespace) {
    unsafe {
        if !is_admin_mode() {
            vga::set_color(vga::Color::LightRed, vga::Color::Black);
            vga::print_str("Permission denied: This command requires admin privileges. Use 'please <command>'.\n");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            return;
        }

        vga::set_color(vga::Color::White, vga::Color::Black);
        vga::print_str("Rebooting Keira Kernel via PS/2 controller...\n");
        core::arch::asm!(
            "out dx, al",
            in("dx") 0x64u16,
            in("al") 0xFEu8,
            options(nomem, nostack, preserves_flags)
        );

        let null_idt: [u8; 6] = [0; 6];
        core::arch::asm!(
            "lidt [{}]",
            "int3",
            in(reg) null_idt.as_ptr(),
            options(noreturn)
        );
    }
}
