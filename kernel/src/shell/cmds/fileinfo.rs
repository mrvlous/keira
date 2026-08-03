// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Keira Kernel: Shell Command 'fileinfo'
//!
//! Implementation of the native 'fileinfo' shell command to display detailed FAT16
//! file metadata (size, cluster offset, attribute flags, timestamp).

use crate::fs::fat;
use crate::io::vga;

pub fn run(parts: &mut core::str::SplitWhitespace) {
    let path = parts.next();

    if let Some("-h") | Some("--help") = path {
        vga::print_str("Usage: fileinfo <file_path>\n\n");
        vga::print_str("Description:\n  Inspect detailed FAT16 file metadata including file size in bytes, first cluster index, attribute bitmasks, and write protection status.\n\n");
        vga::print_str("Options:\n  -h, --help    Show this help message and exit\n\n");
        vga::print_str("Examples:\n  fileinfo kernel.bin\n");
        return;
    }

    if let Some(p) = path {
        unsafe {
            let (dir_cluster, name) = match fat::resolve_path(p) {
                Ok(res) => res,
                Err(_) => {
                    vga::print_str("Fileinfo Error: Invalid file path\n");
                    return;
                }
            };

            let entry = match fat::find_entry(name, dir_cluster) {
                Ok(e) => e,
                Err(_) => {
                    vga::print_str("Fileinfo Error: File not found\n");
                    return;
                }
            };

            vga::set_color(vga::Color::LightCyan, vga::Color::Black);
            vga::print_str("KEIRA FILE METADATA INSPECTOR:\n");
            vga::print_str("  Path            : ");
            vga::set_color(vga::Color::White, vga::Color::Black);
            vga::print_str(p);
            vga::print_str("\n");

            vga::set_color(vga::Color::LightCyan, vga::Color::Black);
            vga::print_str("  File Size       : ");
            vga::set_color(vga::Color::White, vga::Color::Black);
            vga::print_u64(entry.entry.file_size as u64);
            vga::print_str(" Bytes\n");

            vga::set_color(vga::Color::LightCyan, vga::Color::Black);
            vga::print_str("  First Cluster   : 0x");
            vga::set_color(vga::Color::White, vga::Color::Black);
            vga::print_hex(entry.entry.first_cluster_lo as u64);
            vga::print_str("\n");

            vga::set_color(vga::Color::LightCyan, vga::Color::Black);
            vga::print_str("  Attribute Flags : 0x");
            vga::set_color(vga::Color::White, vga::Color::Black);
            vga::print_hex(entry.entry.attr as u64);
            if (entry.entry.attr & 0x01) != 0 {
                vga::print_str(" [READ-ONLY]");
            } else {
                vga::print_str(" [READ-WRITE]");
            }
            if (entry.entry.attr & 0x10) != 0 {
                vga::print_str(" [DIRECTORY]");
            } else {
                vga::print_str(" [REGULAR FILE]");
            }
            vga::print_str("\n");

            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        }
    } else {
        vga::print_str("Usage: fileinfo <file_path>\n");
    }
}
