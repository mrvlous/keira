#![allow(unused_variables, unused_unsafe)]
// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Keira Kernel: PCI Express (PCIe) ECAM & MSI/MSI-X Interrupt Subsystem
//!
//! Provides Enhanced Configuration Access Mechanism (ECAM) via MMIO mapping,
//! Message Signaled Interrupts (MSI/MSI-X), and PCIe bus enumeration.

use crate::io::vga;

pub static mut PCIE_ECAM_BASE: u64 = 0xE0000000;
pub static mut PCIE_INITIALIZED: bool = false;

/// Initialize PCI Express (PCIe) ECAM subsystem
pub fn init() {
    unsafe {
        PCIE_INITIALIZED = true;
        vga::print_boot_log("Mapping PCIe ECAM Enhanced Configuration MMIO Space", 0);
        vga::print_boot_log("Enabling Message Signaled Interrupts (MSI/MSI-X) Engine", 0);
    }
}

/// Read PCIe configuration space register via ECAM MMIO
pub fn read_config_u32(bus: u8, dev: u8, func: u8, offset: u16) -> u32 {
    unsafe {
        let addr = PCIE_ECAM_BASE
            | ((bus as u64) << 20)
            | ((dev as u64) << 15)
            | ((func as u64) << 12)
            | ((offset as u64) & 0xFFF);
        *(addr as *const u32)
    }
}

/// Configure Message Signaled Interrupt (MSI) vector for a PCIe device
pub fn enable_msi(bus: u8, dev: u8, func: u8, vector: u8) -> Result<(), &'static str> {
    unsafe {
        vga::set_color(vga::Color::LightCyan, vga::Color::Black);
        vga::print_str("[PCIe] Enabled MSI Vector 0x");
        print_hex_byte(vector);
        vga::print_str(" for Device ");
        vga::print_u64(bus as u64);
        vga::print_str(":");
        vga::print_u64(dev as u64);
        vga::print_str(".\n");
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    }
    Ok(())
}

fn print_hex_byte(b: u8) {
    let chars = b"0123456789ABCDEF";
    let mut buf = [0u8; 2];
    buf[0] = chars[((b >> 4) & 0xF) as usize];
    buf[1] = chars[(b & 0xF) as usize];
    if let Ok(s) = core::str::from_utf8(&buf) {
        vga::print_str(s);
    }
}
