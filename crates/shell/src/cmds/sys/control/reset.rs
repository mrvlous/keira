// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Implementation of the 'reset' shell command.

use keira_io::vga;

pub fn run(parts: &mut core::str::SplitWhitespace) {
    if let Some("-h") | Some("--help") = parts.next() {
        {
            vga::print_str("Usage: reset\n\n");
            vga::print_str("Description:\n  Reboot the system via hardware reset.\n");
        }
        return;
    }

    unsafe {
        vga::set_color(vga::Color::White, vga::Color::Black);
        vga::print_str("Rebooting Keira Kernel via PS/2 controller...\n");
        #[cfg(target_os = "none")]
        {
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
}
