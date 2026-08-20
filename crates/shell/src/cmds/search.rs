// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

#![allow(unused_variables, unused_unsafe, static_mut_refs)]

//!
//! Implementation of the 'search' shell command to locate matching string patterns in files or pipe streams.

use keira_io::vga;

static mut SEARCH_FILE_BUF: [u8; 8192] = [0u8; 8192];

pub fn run(parts: &mut core::str::SplitWhitespace) {
    unsafe {
        let pattern = match parts.next() {
            Some("-h") | Some("--help") => {
                vga::print_str("Usage: search <pattern> [filename]\n\n");
                vga::print_str("Description:\n  Search for lines matching pattern in a file or from stdin pipe stream.\n\n");
                vga::print_str("Options:\n  -h, --help    Show this help message and exit\n\n");
                vga::print_str("Examples:\n  search main kernel.c\n  list | search bin\n");
                return;
            }
            Some(p) => p,
            None => {
                vga::print_str("Usage: search <pattern> [filename]\n");
                return;
            }
        };

        let filename = parts.next();

        let content: &[u8] = if let Some(file) = filename {
            match keira_fs::vfs::read_file(file, &mut SEARCH_FILE_BUF) {
                Ok(len) => &SEARCH_FILE_BUF[..len],
                Err(e) => {
                    vga::print_str("Error reading file: ");
                    vga::print_str(e);
                    vga::print_str("\n");
                    return;
                }
            }
        } else if keira_io::vga::PIPE_ACTIVE {
            &keira_io::vga::PIPE_BUFFER[..keira_io::vga::PIPE_LEN]
        } else {
            vga::print_str("Error: No input file or pipe stream provided.\n");
            vga::print_str("Usage: search <pattern> [filename]\n");
            return;
        };

        if let Ok(text) = core::str::from_utf8(content) {
            for line in text.lines() {
                if line.contains(pattern) {
                    vga::print_str(line);
                    vga::print_str("\n");
                }
            }
        } else {
            vga::print_str("Error: Input contains invalid UTF-8 encoding\n");
        }
    }
}
