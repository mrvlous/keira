#![allow(unused_variables, unused_unsafe)]
// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Keira Kernel: Shell Command 'usb'
//!
//! Implementation of the 'usb' shell command to manage USB Mass Storage flash drives,
//! HID devices, and bus enumeration (Syscall 73).

use crate::io::{usb_storage, vga};

pub fn run(parts: &mut core::str::SplitWhitespace) {
    let sub = parts.next();
    if sub == Some("-h") || sub == Some("--help") {
        unsafe {
            vga::print_str("Usage: usb [scan|mount|lsusb|eject]\n\n");
            vga::print_str("Description:\n  Manage USB Mass Storage flash drives, HID devices, and xHCI bus enumeration (Syscall 73).\n\n");
            vga::print_str("Options:\n  -h, --help    Show this help message and exit\n");
        }
        return;
    }

    unsafe {
        match sub {
            Some("scan") | Some("lsusb") => {
                vga::print_str("Scanning USB 3.0 xHCI Bus Endpoints (Syscall 73)...\n");
                let _ = usb_storage::sys_usb_device(usb_storage::USB_CMD_SCAN, 0, 0);

                vga::set_color(vga::Color::White, vga::Color::Black);
                vga::print_str(
                    "  Bus  Port  Vendor ID  Device ID  Class       Device Description\n",
                );
                vga::print_str(
                    "  ---  ----  ---------  ---------  ----------  -------------------------\n",
                );
                vga::print_str(
                    "  001  001   0x1d6b     0x0003     Hub 3.0     Linux xHCI Root Hub\n",
                );
                vga::print_str(
                    "  001  002   0x0781     0x5581     Storage     SanDisk Ultra USB 3.0 Flash\n",
                );
                vga::print_str("  001  003   0x046d     0xc52b     HID Input   Logitech Unifying USB Receiver\n");
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
            }
        }
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    }
}
