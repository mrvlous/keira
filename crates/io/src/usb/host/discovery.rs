// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! USB Host Controller (UHCI, OHCI, EHCI, xHCI) detection and bus enumeration.

use crate::bus::pci;

/// Information descriptor for a detected USB Host Controller.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct UsbControllerInfo {
    /// PCI bus index.
    pub bus: u8,
    /// PCI slot index.
    pub slot: u8,
    /// PCI function index.
    pub func: u8,
    /// Programming interface type (0x00 = UHCI, 0x10 = OHCI, 0x20 = EHCI, 0x30 = xHCI).
    pub interface_type: u8,
    /// Base Address Register 0 physical address.
    pub bar0: u32,
    /// Hardware vendor identifier.
    pub vendor_id: u16,
    /// Hardware device identifier.
    pub device_id: u16,
}

/// Array of discovered USB host controllers on the PCI bus.
pub static mut USB_CONTROLLERS: [Option<UsbControllerInfo>; 8] = [None; 8];
/// Number of discovered USB host controllers.
pub static mut USB_CONTROLLER_COUNT: usize = 0;
/// Flag indicating whether USB controller enumeration has completed.
pub static mut USB_INITIALIZED: bool = false;

/// Scans PCI bus for USB host controllers and registers detected hardware.
pub fn init() {
    unsafe {
        let controllers = &mut *core::ptr::addr_of_mut!(USB_CONTROLLERS);
        *controllers = [None; 8];
        USB_CONTROLLER_COUNT = 0;
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
                        controllers[USB_CONTROLLER_COUNT] = Some(UsbControllerInfo {
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
