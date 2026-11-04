// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Panic handler implementation for freestanding `#![no_std]` execution.

#[cfg(target_os = "none")]
use core::panic::PanicInfo;
#[cfg(target_os = "none")]
use keira_io::serial;
#[cfg(target_os = "none")]
use keira_io::vga;

/// Kernel panic handler: prints panic diagnostics to serial COM1 and VGA console and halts CPU.
#[cfg(target_os = "none")]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // 1. Output to COM1 Serial
    serial::print_str("\r\nKernel panic - not syncing: ");
    if let Some(message) = info.message().as_str() {
        serial::print_str(message);
    } else {
        serial::print_str("fatal execution failure");
    }
    serial::print_str("\r\n");

    serial::print_str("CPU: 0 PID: 1 Comm: init 0.8.0 #1\r\n");
    if let Some(location) = info.location() {
        serial::print_str("Location: ");
        serial::print_str(location.file());
        serial::print_str(":");
        serial::print_u64(location.line() as u64);
        serial::print_str("\r\n");
    }
    serial::print_str("---[ end Kernel panic - not syncing ]---\r\n");

    // 2. Output to VGA text console (using Linux console palette on black background)
    vga::set_color(vga::Color::LightRed, vga::Color::Black);
    vga::print_str("\nKernel panic - not syncing: ");
    if let Some(message) = info.message().as_str() {
        vga::print_str(message);
    } else {
        vga::print_str("fatal execution failure");
    }
    vga::print_str("\n");

    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    vga::print_str("CPU: 0 PID: 1 Comm: init 0.8.0 #1\n");
    if let Some(location) = info.location() {
        vga::print_str("Location: ");
        vga::print_str(location.file());
        vga::print_str(":");
        vga::print_u64(location.line() as u64);
        vga::print_str("\n");
    }
    vga::print_str("---[ end Kernel panic - not syncing ]---\n");

    loop {
        unsafe {
            core::arch::asm!("cli");
            core::arch::asm!("hlt");
        }
    }
}
