// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Legacy PCI I/O port configuration space access primitives (ports 0xCF8/0xCFC).

use keira_arch::cpu::{inl, outl};

/// I/O port address for PCI configuration address register.
pub const PCI_CONFIG_ADDR: u16 = 0xCF8;

/// I/O port address for PCI configuration data register.
pub const PCI_CONFIG_DATA: u16 = 0xCFC;

/// Reads a 32-bit dword from PCI configuration space.
///
/// # Safety
///
/// Direct hardware I/O to ports 0xCF8 and 0xCFC.
pub unsafe fn pci_read_config_u32(bus: u8, slot: u8, func: u8, offset: u8) -> u32 {
    let address = ((bus as u32) << 16)
        | ((slot as u32) << 11)
        | ((func as u32) << 8)
        | ((offset as u32) & 0xFC)
        | 0x8000_0000;

    outl(PCI_CONFIG_ADDR, address);
    inl(PCI_CONFIG_DATA)
}

/// Writes a 32-bit dword to PCI configuration space.
///
/// # Safety
///
/// Direct hardware I/O to ports 0xCF8 and 0xCFC.
pub unsafe fn pci_write_config_u32(bus: u8, slot: u8, func: u8, offset: u8, val: u32) {
    let address = ((bus as u32) << 16)
        | ((slot as u32) << 11)
        | ((func as u32) << 8)
        | ((offset as u32) & 0xFC)
        | 0x8000_0000;

    outl(PCI_CONFIG_ADDR, address);
    outl(PCI_CONFIG_DATA, val);
}

/// Helper function to retrieve the 16-bit Vendor ID of a device function.
///
/// # Safety
///
/// Reads PCI configuration dword at offset 0.
pub unsafe fn get_vendor_id(bus: u8, slot: u8, func: u8) -> u16 {
    let val = pci_read_config_u32(bus, slot, func, 0);
    (val & 0xFFFF) as u16
}
