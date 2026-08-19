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
//! Implementation of the file move/rename command.

use crate::fs::vfs;

static mut MOVE_BUFFER: [u8; 65536] = [0; 65536];

use crate::io::vga;

pub fn run(parts: &mut core::str::SplitWhitespace) {
    let src = match parts.next() {
        Some("-h") | Some("--help") => {
            vga::print_str("Usage: move <src_file> <dest_file>\n\n");
            vga::print_str("Description:\n  Move or rename a file from source path to destination path on FAT16 storage.\n\n");
            vga::print_str("Options:\n  -h, --help    Show this help message and exit\n\n");
            vga::print_str("Examples:\n  move file.txt file_renamed.txt\n");
            return;
        }
        Some(s) => s,
        None => {
            vga::print_str("Usage: move <src_file> <dest_file>\n");
            return;
        }
    };

    let dest = match parts.next() {
        Some(d) => d,
        None => {
            vga::print_str("Usage: move <src_file> <dest_file>\n");
            return;
        }
    };

    unsafe {
        let move_buf = &mut *core::ptr::addr_of_mut!(MOVE_BUFFER);
        // Read source file content into the static move buffer
        match vfs::read_file(src, move_buf) {
            Ok(size) => {
                // Try to create the destination file (if it already exists, we will overwrite it)
                let _ = vfs::create_file(dest);

                match vfs::write_file(dest, &move_buf[..size]) {
                    Ok(_) => {
                        // Delete the source file after a successful copy
                        match vfs::remove_entry(src) {
                            Ok(_) => {
                                vga::print_str("File moved successfully.\n");
                            }
                            Err(e) => {
                                vga::print_str("move warning: Copied successfully, but failed to remove source: ");
                                vga::print_str(e);
                                vga::print_str("\n");
                            }
                        }
                    }
                    Err(e) => {
                        vga::print_str("move error: Failed to write destination: ");
                        vga::print_str(e);
                        vga::print_str("\n");
                    }
                }
            }
            Err(e) => {
                vga::print_str("move error: Failed to read source: ");
                vga::print_str(e);
                vga::print_str("\n");
            }
        }
    }
}
