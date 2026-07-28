#![allow(unused_variables, unused_unsafe)]
// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Keira Kernel: Shell Command 'list'
//!
//! Implementation of the 'list' shell command.

use crate::io::vga;

pub fn run(parts: &mut core::str::SplitWhitespace) {
    unsafe {
        let mut show_all = false;
        let mut path_arg = None;

        for part in parts {
            if part == "-a" || part == "-all" || part == "--all" {
                show_all = true;
            } else if part.starts_with('-') {
                vga::set_color(vga::Color::LightRed, vga::Color::Black);
                vga::print_str("Warning: Unknown option '");
                vga::print_str(part);
                vga::print_str("'. Supported options: -a, -all.\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            } else {
                path_arg = Some(part);
            }
        }

        unsafe {
            let target_cluster = if let Some(path) = path_arg {
                match crate::fs::fat::get_dir_cluster(path) {
                    Ok(cluster) => cluster,
                    Err(e) => {
                        vga::set_color(vga::Color::LightRed, vga::Color::Black);
                        vga::print_str("list: ");
                        vga::print_str(e);
                        vga::print_str("\n");
                        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                        return;
                    }
                }
            } else {
                crate::fs::fat::CURRENT_DIR_CLUSTER
            };

            if let Err(_) = crate::fs::fat::list_files_in_dir(target_cluster, show_all) {
                // Fallback to initrd RAM disk listing
                crate::fs::tar::list_files();
            }
        }
    }
}
