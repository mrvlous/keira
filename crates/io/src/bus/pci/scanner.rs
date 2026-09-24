// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! PCI bus enumerator and device discovery loop.

use super::config::{get_vendor_id, pci_read_config_u32};
use super::device::{PciDevice, PCI_DEVICES, PCI_DEVICE_COUNT};

/// Scans the PCI bus hierarchy for present hardware devices.
pub fn init() {
    unsafe {
        PCI_DEVICE_COUNT = 0;
        PCI_DEVICES = [None; 32];

        for bus in 0..8 {
            for slot in 0..32 {
                let vendor_id = get_vendor_id(bus, slot, 0);
                if vendor_id == 0xFFFF || vendor_id == 0x0000 {
                    continue;
                }

                let header_type_reg = pci_read_config_u32(bus, slot, 0, 0x0C);
                let header_type = ((header_type_reg >> 16) & 0xFF) as u8;
                let multi_func = (header_type & 0x80) != 0;

                let functions_to_scan = if multi_func { 8 } else { 1 };

                for func in 0..functions_to_scan {
                    let v_id = get_vendor_id(bus, slot, func);
                    if v_id == 0xFFFF || v_id == 0x0000 {
                        continue;
                    }

                    let dev_id_reg = pci_read_config_u32(bus, slot, func, 0);
                    let device_id = (dev_id_reg >> 16) as u16;

                    let class_reg = pci_read_config_u32(bus, slot, func, 8);
                    let class_code = ((class_reg >> 24) & 0xFF) as u8;
                    let subclass = ((class_reg >> 16) & 0xFF) as u8;
                    let prog_if = ((class_reg >> 8) & 0xFF) as u8;

                    let bar5 = pci_read_config_u32(bus, slot, func, 0x24);

                    if PCI_DEVICE_COUNT < 32 {
                        PCI_DEVICES[PCI_DEVICE_COUNT] = Some(PciDevice {
                            bus,
                            slot,
                            func,
                            vendor_id: v_id,
                            device_id,
                            class_code,
                            subclass,
                            prog_if,
                            bar5,
                        });
                        PCI_DEVICE_COUNT += 1;
                    }
                }
            }
        }
    }
}
