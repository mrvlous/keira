#![allow(unused_variables, unused_unsafe)]
// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Keira Kernel: Shell Command 'ramdisk'
//!
//! Implementation of the 'ramdisk' shell command.

use crate::io::vga;
use crate::shell::executor::*;

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
        let sub = parts.next();
        match sub {
            Some("-h") | Some("--help") => {
                vga::print_str("Usage: ramdisk create <size_kb>\n\n");
                vga::print_str("Description:\n  Dynamically allocate a RAM Disk in memory, auto-format as FAT16, and register block device ram0.\n\n");
                vga::print_str("Options:\n  -h, --help    Show this help message and exit\n\n");
                vga::print_str("Examples:\n  ramdisk create 1024\n");
                return;
            }
            Some("create") => {
                let size_str = parts.next();
                match size_str {
                    None => {
                        vga::print_str(
                            "Usage: ramdisk create <size_kb> (e.g. ramdisk create 1024)\n",
                        );
                    }
                    Some(s) => {
                        let mut size_kb = 0u32;
                        let mut valid = true;
                        for c in s.chars() {
                            if c.is_ascii_digit() {
                                size_kb = size_kb * 10 + (c as u32 - '0' as u32);
                            } else {
                                valid = false;
                                break;
                            }
                        }
                        if !valid || size_kb == 0 {
                            vga::print_str("Error: Invalid size parameter\n");
                        } else {
                            vga::print_str("Creating ram0 (");
                            vga::print_u64(size_kb as u64);
                            vga::print_str(" KB)...\n");
                            match crate::io::ramdisk::create_ramdisk(size_kb) {
                                Ok(_) => {
                                    vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                                    vga::print_str("Ramdisk 'ram0' successfully created & auto-formatted as FAT16.\n");
                                    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                                }
                                Err(e) => {
                                    vga::set_color(vga::Color::LightRed, vga::Color::Black);
                                    vga::print_str("Failed to create ramdisk: ");
                                    vga::print_str(e);
                                    vga::print_str("\n");
                                    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                                }
                            }
                        }
                    }
                }
            }
            _ => {
                vga::print_str("Usage: ramdisk create <size_kb> (up to 4096 KB)\n");
            }
        }
    }
}
