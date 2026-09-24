// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! PCIe Enhanced Configuration Access Mechanism (ECAM) base address and MMIO access.

/// Base physical address of the PCIe ECAM memory mapped configuration region.
pub static mut PCIE_ECAM_BASE: u64 = 0xE0000000;

/// Flag indicating whether PCIe ECAM configuration has been initialized.
pub static mut PCIE_INITIALIZED: bool = false;

/// Initializes the PCI Express (PCIe) ECAM subsystem.
pub fn init() {
    unsafe {
        PCIE_INITIALIZED = true;
    }
}

/// Reads a PCIe configuration space register via ECAM MMIO.
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
