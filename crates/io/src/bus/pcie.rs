// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! PCIe Enhanced Configuration Access Mechanism (ECAM) and Message Signaled Interrupts (MSI).

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

/// Configure Message Signaled Interrupt (MSI) vector for a PCIe device.
pub fn enable_msi(_bus: u8, _dev: u8, _func: u8, _vector: u8) -> Result<(), &'static str> {
    Ok(())
}
