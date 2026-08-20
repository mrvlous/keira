// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! USB Host Controller (UHCI, OHCI, EHCI, xHCI) detection and bus enumeration.

use crate::bus::pci;

/// Information descriptor for a detected USB Host Controller.
#[derive(Copy, Clone, Debug)]
pub struct UsbControllerInfo {
    pub bus: u8,
    pub slot: u8,
    pub func: u8,
    pub interface_type: u8, // 0x00 = UHCI, 0x10 = OHCI, 0x20 = EHCI, 0x30 = xHCI
    pub bar0: u32,
    pub vendor_id: u16,
    pub device_id: u16,
}

pub static mut USB_CONTROLLERS: [Option<UsbControllerInfo>; 8] = [None; 8];
pub static mut USB_CONTROLLER_COUNT: usize = 0;
pub static mut USB_INITIALIZED: bool = false;

/// Scan PCI bus for USB host controllers.
pub fn init() {
    unsafe {
        USB_CONTROLLER_COUNT = 0;
        USB_CONTROLLERS = [None; 8];
        USB_INITIALIZED = true;

        for bus in 0..16 {
            for slot in 0..32 {
                let func = 0;
                let reg0 = pci::pci_read_config_u32(bus, slot, func, 0x00);
                let vendor = (reg0 & 0xFFFF) as u16;
                if vendor == 0xFFFF || vendor == 0x0000 {
                    continue;
                }

                let class_reg = pci::pci_read_config_u32(bus, slot, func, 0x08);
                let class_code = ((class_reg >> 24) & 0xFF) as u8;
                let subclass_code = ((class_reg >> 16) & 0xFF) as u8;

                if class_code == 0x0C && subclass_code == 0x03 {
                    let prog_if = ((class_reg >> 8) & 0xFF) as u8;
                    let device_id = (reg0 >> 16) as u16;
                    let bar0 = pci::pci_read_config_u32(bus, slot, func, 0x10);

                    if USB_CONTROLLER_COUNT < 8 {
                        USB_CONTROLLERS[USB_CONTROLLER_COUNT] = Some(UsbControllerInfo {
                            bus,
                            slot,
                            func,
                            interface_type: prog_if,
                            bar0,
                            vendor_id: vendor,
                            device_id,
                        });
                        USB_CONTROLLER_COUNT += 1;
                    }
                }
            }
        }
    }
}
