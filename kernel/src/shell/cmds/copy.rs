#![allow(unused_variables, unused_unsafe)]
// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Keira Kernel: Shell Command 'copy'
//!
//! Implementation of the file copy command.

use crate::fs::vfs;

static mut COPY_BUFFER: [u8; 65536] = [0; 65536];

use crate::io::vga;

pub fn run(parts: &mut core::str::SplitWhitespace) {
    let src = match parts.next() {
        Some(s) => s,
        None => {
            vga::print_str("Usage: copy <src_file> <dest_file>\n");
            return;
        }
    };

    let dest = match parts.next() {
        Some(d) => d,
        None => {
            vga::print_str("Usage: copy <src_file> <dest_file>\n");
            return;
        }
    };

    unsafe {
        let copy_buf = &mut *core::ptr::addr_of_mut!(COPY_BUFFER);
        // Read source file content into the static copy buffer
        match vfs::read_file(src, copy_buf) {
            Ok(size) => {
                // Try to create the destination file (if it already exists, we will overwrite it)
                let _ = vfs::create_file(dest);

                match vfs::write_file(dest, &copy_buf[..size]) {
                    Ok(_) => {
                        vga::print_str("File copied successfully.\n");
                    }
                    Err(e) => {
                        vga::print_str("copy error: Failed to write destination: ");
                        vga::print_str(e);
                        vga::print_str("\n");
                    }
                }
            }
            Err(e) => {
                vga::print_str("copy error: Failed to read source: ");
                vga::print_str(e);
                vga::print_str("\n");
            }
        }
    }
}
