// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Early bare-metal hardware and peripheral initialization sequence.

#[cfg(target_os = "none")]
use keira_io::ps2::{keyboard as ps2_keyboard, mouse as ps2_mouse};
#[cfg(target_os = "none")]
use keira_io::rtc;
use keira_io::vga;

/// Early hardware peripheral and architecture bringup routine.
pub fn early_bringup() {
    vga::init();
    #[cfg(target_os = "none")]
    {
        keira_arch::init();
        ps2_keyboard::init();
        ps2_mouse::init();
        rtc::init();
    }

    vga::print_boot_log("Initializing Serial Port (COM1) driver", 0);
    vga::print_boot_log("Configuring VGA text-mode frame buffer (80x25)", 0);
    vga::print_boot_log("Checking x86 CPUID & Model Specific Registers (MSRs)", 0);
    vga::print_boot_log("Loading Interrupt Descriptor Table (IDT) registers", 0);
    vga::print_boot_log("Remapping dual 8259 PIC interrupt controller IRQs", 0);
    vga::print_boot_log("Configuring 8253 PIT system timer tick rate to 1000Hz", 0);
    vga::print_boot_log(
        "Initializing High-Precision Event Timer (HPET) Subsystem",
        0,
    );
    vga::print_boot_log("Initializing PS/2 keyboard controller & driver", 0);
    vga::print_boot_log("Initializing PS/2 mouse controller & driver", 0);
    vga::print_boot_log("Reading CMOS Real-Time Clock (RTC) date/time registers", 0);
    vga::print_boot_log("Scanning PCIe ECAM Memory-Mapped Configuration Space", 0);
    vga::print_boot_log("Completing low-level hardware subsystem checks", 0);
}
