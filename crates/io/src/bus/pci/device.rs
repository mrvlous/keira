// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! PCI device representation, device table storage, and class translation.

/// Structure representing a detected PCI hardware device.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct PciDevice {
    /// PCI bus number (0..255).
    pub bus: u8,
    /// Device slot number (0..31).
    pub slot: u8,
    /// Function number (0..7).
    pub func: u8,
    /// 16-bit Vendor Identifier.
    pub vendor_id: u16,
    /// 16-bit Device Identifier.
    pub device_id: u16,
    /// Base class code.
    pub class_code: u8,
    /// Subclass code.
    pub subclass: u8,
    /// Programming interface register.
    pub prog_if: u8,
    /// Base Address Register 5 (typically AHCI or memory-mapped I/O base).
    pub bar5: u32,
}

/// Global table of detected PCI hardware devices.
pub static mut PCI_DEVICES: [Option<PciDevice>; 32] = [None; 32];

/// Number of active entries populated in `PCI_DEVICES`.
pub static mut PCI_DEVICE_COUNT: usize = 0;

/// Translates PCI class and subclass codes into descriptive human-readable strings.
pub fn pci_class_to_str(class: u8, subclass: u8) -> &'static str {
    match class {
        0x00 => "Unclassified",
        0x01 => match subclass {
            0x00 => "SCSI Controller",
            0x01 => "IDE Controller",
            0x02 => "Floppy Controller",
            0x03 => "IPI Controller",
            0x04 => "RAID Controller",
            0x05 => "ATA Controller",
            0x06 => "SATA Controller (AHCI)",
            0x07 => "SAS Controller",
            0x08 => "NVMe Controller",
            _ => "Storage Controller",
        },
        0x02 => "Network Controller",
        0x03 => "Display Controller (VGA)",
        0x04 => "Multimedia Controller",
        0x05 => "Memory Controller",
        0x06 => "Bridge Device",
        0x07 => "Simple Comm Controller",
        0x08 => "System Base Peripheral",
        0x09 => "Input Device Controller",
        0x0A => "Docking Station",
        0x0B => "Processor",
        0x0C => "Serial Bus (USB/SMBus)",
        0x0D => "Wireless Controller",
        0x0E => "Intelligent Controller",
        0x0F => "Satellite Comm Controller",
        0x10 => "Encryption Controller",
        0x11 => "Signal Processing Controller",
        _ => "Unknown Device Class",
    }
}
