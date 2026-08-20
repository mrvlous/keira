// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Panic handler implementation for freestanding `#![no_std]` execution.

use core::panic::PanicInfo;
use keira_io::serial;
use keira_io::vga;

/// Kernel panic handler : prints panic info to serial and VGA and halts.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial::print_str("\n\n!!! KERNEL PANIC !!!\n");

    if let Some(message) = info.message().as_str() {
        serial::print_str("Message: ");
        serial::print_str(message);
        serial::print_str("\n");
    }

    if let Some(location) = info.location() {
        serial::print_str("Location: ");
        serial::print_str(location.file());
        serial::print_str(" Line: ");
        let mut line = location.line();
        if line == 0 {
            serial::putchar(b'0');
        } else {
            let mut buf = [0u8; 10];
            let mut i = 9;
            while line > 0 {
                buf[i] = b'0' + (line % 10) as u8;
                line /= 10;
                if i == 0 {
                    break;
                }
                i -= 1;
            }
            serial::print(&buf[(i + 1)..=9]);
        }
        serial::print_str("\n");
    }

    serial::print_str("System halted.\n");

    vga::set_color(vga::Color::White, vga::Color::Blue);
    vga::init();

    vga::print_str("\n");
    vga::print_str("  KEIRA KERNEL PANIC\n\n");

    if let Some(message) = info.message().as_str() {
        vga::print_str("  Message: ");
        vga::print_str(message);
        vga::print_str("\n");
    } else {
        vga::print_str("  Message: Undefined kernel execution failure.\n");
    }

    if let Some(location) = info.location() {
        vga::print_str("  Location: ");
        vga::print_str(location.file());
        vga::print_str(":");
        vga::print_u64(location.line() as u64);
        vga::print_str("\n");
    }
    vga::print_str("\n");

    vga::print_str("  A fatal error has occurred and the system was halted to prevent damage.\n");
    vga::print_str("  Please restart your computer or emulator.\n");

    loop {
        unsafe {
            core::arch::asm!("cli");
            core::arch::asm!("hlt");
        }
    }
}
