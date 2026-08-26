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
//! Implementation of the 'use' shell command.

use crate::executor::*;
use keira_io::vga;

pub fn run(parts: &mut core::str::SplitWhitespace) {
    unsafe {
        unsafe {
            if !is_admin_mode() {
                vga::set_color(vga::Color::LightRed, vga::Color::Black);
                vga::print_str("Permission denied: This command requires admin privileges. Use 'please <command>'.\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                return;
            }
        }
        let dev_name = parts.next();
        match dev_name {
            Some("-h") | Some("--help") => {
                vga::print_str("Usage: use <device_name>\n\n");
                vga::print_str("Description:\n  Mount and activate a block storage device (e.g. ram0, ahci0) as the active FAT16 filesystem volume.\n\n");
                vga::print_str("Options:\n  -h, --help    Show this help message and exit\n\n");
                vga::print_str("Examples:\n  use ram0\n");
            }
            None => {
                vga::print_str("Usage: use <device_name> (e.g. use ram0)\n");
            }
            Some(name) => {
                vga::print_str("Activating ");
                vga::print_str(name);
                vga::print_str("...\n");
                match keira_io::block::mount_device(name) {
                    Ok(_) => unsafe {
                        match keira_fs::fat::init() {
                            Ok(_) => {
                                vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                                vga::print_str(
                                    "Successfully mounted and initialized FAT16 filesystem.\n",
                                );
                                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                            }
                            Err(e) => {
                                vga::set_color(vga::Color::LightRed, vga::Color::Black);
                                vga::print_str(
                                    "Activation failed: Unable to initialize FAT16 on device: ",
                                );
                                vga::print_str(e);
                                vga::print_str("\n");
                                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                            }
                        }
                    },
                    Err(e) => {
                        vga::set_color(vga::Color::LightRed, vga::Color::Black);
                        vga::print_str("Activation failed: ");
                        vga::print_str(e);
                        vga::print_str("\n");
                        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                    }
                }
            }
        }
    }
}
