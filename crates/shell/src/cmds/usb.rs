// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

#![allow(unused_variables, unused_unsafe)]

//!
//! Implementation of the 'usb' shell command to manage USB Mass Storage flash drives,
//! HID devices, and bus enumeration (Syscall 73).

use keira_io::usb::storage as usb_storage;
use keira_io::vga;

fn print_hex_u16(val: u16) {
    let chars = b"0123456789abcdef";
    let b3 = chars[((val >> 12) & 0xF) as usize];
    let b2 = chars[((val >> 8) & 0xF) as usize];
    let b1 = chars[((val >> 4) & 0xF) as usize];
    let b0 = chars[(val & 0xF) as usize];
    let buf = [b3, b2, b1, b0];
    if let Ok(s) = core::str::from_utf8(&buf) {
        unsafe {
            vga::print_str(s);
        }
    }
}

pub fn run(parts: &mut core::str::SplitWhitespace) {
    let sub = parts.next();
    if sub == Some("-h") || sub == Some("--help") {
        unsafe {
            vga::print_str("Usage: usb [scan|mount|lsusb|eject]\n\n");
            vga::print_str("Description:\n  Manage USB Mass Storage flash drives, HID devices, and PCI USB enumeration (Syscall 73).\n\n");
            vga::print_str("Options:\n  -h, --help    Show this help message and exit\n");
        }
        return;
    }

    unsafe {
        match sub {
            Some("scan") | Some("lsusb") => {
                vga::print_str("Scanning PCI Bus for USB Host Controllers (Syscall 73)...\n");
                let _ = usb_storage::sys_usb_device(usb_storage::USB_CMD_SCAN, 0, 0);

                vga::set_color(vga::Color::White, vga::Color::Black);
                vga::print_str("  Bus  Slot  Func  Vendor ID  Device ID  Type\n");
                vga::print_str(
                    "  ---  ----  ----  ---------  ---------  -------------------------\n",
                );
                vga::set_color(vga::Color::White, vga::Color::Black);

                let mut found = false;
                for i in 0..keira_io::pci::PCI_DEVICE_COUNT {
                    if let Some(dev) = keira_io::pci::PCI_DEVICES[i] {
                        if dev.class_code == 0x0C && dev.subclass == 0x03 {
                            found = true;
                            vga::print_str("  ");
                            vga::print_u64(dev.bus as u64);
                            vga::print_str("    ");
                            vga::print_u64(dev.slot as u64);
                            vga::print_str("     ");
                            vga::print_u64(dev.func as u64);
                            vga::print_str("     0x");
                            print_hex_u16(dev.vendor_id);
                            vga::print_str("     0x");
                            print_hex_u16(dev.device_id);
                            vga::print_str("     USB Host Controller\n");
                        }
                    }
                }
                if !found {
                    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                    vga::print_str("  (No PCI USB host controllers detected)\n");
                }
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            }
            Some("mount") => {
                let _ = usb_storage::sys_usb_device(usb_storage::USB_CMD_MOUNT, 0, 0);
            }
            Some("eject") => {
                let _ = usb_storage::sys_usb_device(usb_storage::USB_CMD_EJECT, 0, 0);
            }
            _ => {
                vga::set_color(vga::Color::Yellow, vga::Color::Black);
                vga::print_str("Usage: usb [scan|mount|lsusb|eject]\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            }
        }
    }
}
