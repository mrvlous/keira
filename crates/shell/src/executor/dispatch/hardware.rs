// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Hardware probe queries, numeric display formatting, and RTC data structures.

use keira_io::vga;

/// Re-export of VGA initialization for command routines.
pub fn vga_init() {
    vga::init();
}

/// CMOS Real-Time Clock binary timestamp structure.
#[repr(C)]
pub struct RtcTime {
    pub second: u8,
    pub minute: u8,
    pub hour: u8,
    pub day: u8,
    pub month: u8,
    pub year: u16,
}

/// Print an integer formatted to at least 2 digits with a leading zero if needed.
pub fn print_2digit(n: u64) {
    if n < 10 {
        vga::print_str("0");
    }
    vga::print_u64(n);
}

/// Probe and count connected PCI configuration space devices across all buses and slots.
pub fn count_pci_devices() -> u64 {
    #[cfg(not(target_os = "none"))]
    {
        0
    }
    #[cfg(target_os = "none")]
    {
        let mut count = 0u64;
        for bus in 0..=255u16 {
            for slot in 0..32u8 {
                let address = ((bus as u32) << 16) | ((slot as u32) << 11) | 0x80000000u32;
                unsafe {
                    core::arch::asm!(
                        "out dx, eax",
                        in("dx") 0xCF8u16,
                        in("eax") address,
                        options(nomem, nostack, preserves_flags)
                    );
                    let value: u32;
                    core::arch::asm!(
                        "in eax, dx",
                        out("eax") value,
                        in("dx") 0xCFCu16,
                        options(nomem, nostack, preserves_flags)
                    );
                    if (value & 0xFFFF) != 0xFFFF {
                        count += 1;
                    }
                }
            }
        }
        count
    }
}
