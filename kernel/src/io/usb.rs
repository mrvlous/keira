// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Keira Kernel: USB Host Controller Subsystem (xHCI / EHCI / UHCI / OHCI)
//!
//! Provides PCI enumeration of USB host controllers, USB device descriptor decoding,
//! endpoint data pipe initialization, and bus status querying.

use crate::io::pci;
use crate::io::vga;

#[derive(Copy, Clone)]
pub struct UsbControllerInfo {
    pub bus: u8,
    pub slot: u8,
    pub func: u8,
    pub interface_type: u8, // 0x00: UHCI, 0x10: OHCI, 0x20: EHCI, 0x30: xHCI
    pub bar0: u32,
    pub vendor_id: u16,
    pub device_id: u16,
}

pub static mut USB_CONTROLLERS: [Option<UsbControllerInfo>; 8] = [None; 8];
pub static mut USB_CONTROLLER_COUNT: usize = 0;
pub static mut USB_INITIALIZED: bool = false;

/// Scan PCI bus for USB Host Controllers (Class 0x0C, Subclass 0x03)
pub unsafe fn init_usb_subsystem() {
    USB_CONTROLLER_COUNT = 0;
    USB_INITIALIZED = true;

    for bus in 0..16 {
        for slot in 0..32 {
            for func in 0..8 {
                let reg0 = pci::pci_read_config_u32(bus, slot, func, 0x00);
                let vendor = (reg0 & 0xFFFF) as u16;
                if vendor == 0xFFFF || vendor == 0x0000 {
                    continue;
                }

                let class_reg = pci::pci_read_config_u32(bus, slot, func, 0x08);
                let class_code = ((class_reg >> 24) & 0xFF) as u8;
                let subclass_code = ((class_reg >> 16) & 0xFF) as u8;

                // Serial Bus Controller (0x0C), USB Controller (0x03)
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

/// Print USB Host Controller Subsystem Information
pub unsafe fn print_usb_info() {
    if !USB_INITIALIZED {
        init_usb_subsystem();
    }

    vga::set_color(vga::Color::LightCyan, vga::Color::Black);
    vga::print_str("KEIRA USB HOST CONTROLLER SUBSYSTEM:\n");
    vga::set_color(vga::Color::White, vga::Color::Black);

    vga::print_str("  Total USB Host Controllers Found: ");
    vga::print_u64(USB_CONTROLLER_COUNT as u64);
    vga::print_str("\n\n");

    if USB_CONTROLLER_COUNT == 0 {
        vga::set_color(vga::Color::Yellow, vga::Color::Black);
        vga::print_str("  No USB Host Controllers detected on PCI bus 0..15.\n");
    } else {
        for i in 0..USB_CONTROLLER_COUNT {
            if let Some(ctrl) = USB_CONTROLLERS[i] {
                vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                vga::print_str("  [USB Controller #");
                vga::print_u64(i as u64);
                vga::print_str("]\n");

                vga::set_color(vga::Color::White, vga::Color::Black);
                vga::print_str("    PCI Location   : ");
                vga::print_u64(ctrl.bus as u64);
                vga::print_str(":");
                vga::print_u64(ctrl.slot as u64);
                vga::print_str(".");
                vga::print_u64(ctrl.func as u64);

                vga::print_str("\n    Interface Type : ");
                match ctrl.interface_type {
                    0x00 => vga::print_str("UHCI (USB 1.1 Universal Host Controller)\n"),
                    0x10 => vga::print_str("OHCI (USB 1.1 Open Host Controller)\n"),
                    0x20 => vga::print_str("EHCI (USB 2.0 Enhanced Host Controller)\n"),
                    0x30 => vga::print_str("xHCI (USB 3.0 Extensible Host Controller)\n"),
                    _ => vga::print_str("Custom USB Controller Interface\n"),
                }

                vga::print_str("    Vendor / Device: 0x");
                vga::print_hex(ctrl.vendor_id as u64);
                vga::print_str(" / 0x");
                vga::print_hex(ctrl.device_id as u64);
                vga::print_str("\n    MMIO BAR0      : 0x");
                vga::print_hex(ctrl.bar0 as u64);
                vga::print_str("\n\n");
            }
        }
    }
    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
}
