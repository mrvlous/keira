// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Unit tests for PCI class code translation and device representations.

use super::device::{pci_class_to_str, PciDevice};

#[test]
fn test_pci_class_description() {
    assert_eq!(pci_class_to_str(0x01, 0x06), "SATA Controller (AHCI)");
    assert_eq!(pci_class_to_str(0x01, 0x08), "NVMe Controller");
    assert_eq!(pci_class_to_str(0x02, 0x00), "Network Controller");
    assert_eq!(pci_class_to_str(0x03, 0x00), "Display Controller (VGA)");
    assert_eq!(pci_class_to_str(0x0C, 0x03), "Serial Bus (USB/SMBus)");
    assert_eq!(pci_class_to_str(0xFF, 0xFF), "Unknown Device Class");
}

#[test]
fn test_pci_device_instantiation() {
    let dev = PciDevice {
        bus: 0,
        slot: 1,
        func: 0,
        vendor_id: 0x8086,
        device_id: 0x100E,
        class_code: 0x02,
        subclass: 0x00,
        prog_if: 0x00,
        bar5: 0xFEB0_0000,
    };
    assert_eq!(dev.vendor_id, 0x8086);
    assert_eq!(dev.bar5, 0xFEB0_0000);
}
