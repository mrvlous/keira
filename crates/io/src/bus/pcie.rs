// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! PCIe Enhanced Configuration Access Mechanism (ECAM) and Message Signaled Interrupts (MSI).

use super::pci::{pci_read_config_u32, pci_write_config_u32};

pub static mut PCIE_ECAM_BASE: u64 = 0xE0000000;
pub static mut PCIE_INITIALIZED: bool = false;

/// Initialize PCI Express (PCIe) ECAM subsystem.
pub fn init() {
    unsafe {
        PCIE_INITIALIZED = true;
    }
}

/// Read PCIe configuration space register via ECAM MMIO.
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

/// Find the offset of the MSI Capability structure (Cap ID 0x05) in PCI configuration space.
pub fn find_msi_capability(bus: u8, dev: u8, func: u8) -> Option<u8> {
    unsafe {
        let status_cmd = pci_read_config_u32(bus, dev, func, 0x04);
        let status = (status_cmd >> 16) as u16;

        // Check Capabilities List bit (bit 4 of Status register)
        if (status & (1 << 4)) == 0 {
            return None;
        }

        let mut cap_ptr = (pci_read_config_u32(bus, dev, func, 0x34) & 0xFC) as u8;
        let mut hops = 0;

        while cap_ptr >= 0x40 && hops < 48 {
            let cap_hdr = pci_read_config_u32(bus, dev, func, cap_ptr);
            let cap_id = (cap_hdr & 0xFF) as u8;
            let next_ptr = ((cap_hdr >> 8) & 0xFC) as u8;

            if cap_id == 0x05 {
                return Some(cap_ptr);
            }

            cap_ptr = next_ptr;
            hops += 1;
        }

        None
    }
}

/// Configure Message Signaled Interrupt (MSI) vector for a PCIe/PCI device.
pub fn enable_msi(bus: u8, dev: u8, func: u8, vector: u8) -> Result<(), &'static str> {
    enable_msi_target(bus, dev, func, vector, 0)
}

/// Configure Message Signaled Interrupt (MSI) with specific destination APIC CPU ID.
pub fn enable_msi_target(
    bus: u8,
    dev: u8,
    func: u8,
    vector: u8,
    dest_apic_id: u8,
) -> Result<(), &'static str> {
    let cap_ptr =
        find_msi_capability(bus, dev, func).ok_or("Device does not support PCI MSI capability")?;

    unsafe {
        let cap_hdr = pci_read_config_u32(bus, dev, func, cap_ptr);
        let msg_ctrl = ((cap_hdr >> 16) & 0xFFFF) as u16;
        let is_64bit = (msg_ctrl & (1 << 7)) != 0;

        // x86 Local APIC delivery address format: 0xFEE0_0000 | (dest_apic_id << 12)
        let lapic_addr = 0xFEE0_0000 | ((dest_apic_id as u32) << 12);
        pci_write_config_u32(bus, dev, func, cap_ptr + 4, lapic_addr);

        if is_64bit {
            // Upper 32-bit address (0 on 32-bit APIC systems)
            pci_write_config_u32(bus, dev, func, cap_ptr + 8, 0);

            // Message Data register at cap_ptr + 12 (vector in bits 0..7, delivery mode Fixed)
            let cur_data = pci_read_config_u32(bus, dev, func, cap_ptr + 12);
            let new_data = (cur_data & 0xFFFF_0000) | (vector as u32);
            pci_write_config_u32(bus, dev, func, cap_ptr + 12, new_data);
        } else {
            // Message Data register at cap_ptr + 8
            let cur_data = pci_read_config_u32(bus, dev, func, cap_ptr + 8);
            let new_data = (cur_data & 0xFFFF_0000) | (vector as u32);
            pci_write_config_u32(bus, dev, func, cap_ptr + 8, new_data);
        }

        // Enable MSI: set bit 0 of Message Control Register
        let new_ctrl = msg_ctrl | 0x0001;
        let new_hdr = (cap_hdr & 0x0000_FFFF) | ((new_ctrl as u32) << 16);
        pci_write_config_u32(bus, dev, func, cap_ptr, new_hdr);

        Ok(())
    }
}
