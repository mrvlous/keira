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
//! Implementation of the 'write' shell command to write or append text content to a FAT16 file.

use crate::io::vga;
use crate::shell::executor::*;

pub fn run(parts: &mut core::str::SplitWhitespace) {
    unsafe {
        if !check_write_permission() {
            vga::set_color(vga::Color::LightRed, vga::Color::Black);
            vga::print_str("Permission denied: Non-admin users cannot write outside their home directory. Use 'please' to run as admin.\n");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            return;
        }

        let first_arg = match parts.next() {
            Some("-h") | Some("--help") => {
                vga::print_str("Usage: write [-a|--append|>>] <filename> <text...>\n\n");
                vga::print_str("Description:\n  Write or append text content to a file on FAT16 disk storage.\n\n");
                vga::print_str("Options:\n  -a, --append, >>  Append text stream to existing file end-of-chain\n  -h, --help        Show this help message and exit\n\n");
                vga::print_str("Examples:\n  write note.txt Hello Keira!\n  write -a note.txt New line appended.\n");
                return;
            }
            Some(s) => s,
            None => {
                vga::print_str("Usage: write [-a|--append|>>] <filename> <text>\n");
                return;
            }
        };

        let is_append = first_arg == "-a" || first_arg == "--append" || first_arg == ">>";

        let filename = if is_append {
            match parts.next() {
                Some(s) => s,
                None => {
                    vga::print_str("Usage: write [-a|--append|>>] <filename> <text>\n");
                    return;
                }
            }
        } else {
            first_arg
        };

        // Gather the rest of the arguments as the text content
        let mut text_buf = [0u8; 1024];
        let mut text_len = 0;

        while let Some(part) = parts.next() {
            let part_bytes = part.as_bytes();
            if text_len > 0 && text_len < 1024 {
                text_buf[text_len] = b' ';
                text_len += 1;
            }
            if text_len + part_bytes.len() < 1024 {
                text_buf[text_len..text_len + part_bytes.len()].copy_from_slice(part_bytes);
                text_len += part_bytes.len();
            } else {
                break;
            }
        }

        // Check if file exists, if not, create it first
        let file_exists = if let Ok((dir_cluster, name)) = crate::fs::fat::resolve_path(filename) {
            crate::fs::fat::find_entry(name, dir_cluster).is_ok()
        } else {
            false
        };

        if !file_exists {
            if let Err(e) = crate::fs::fat::create_file(filename) {
                vga::print_str("Error creating file: ");
                vga::print_str(e);
                vga::print_str("\n");
                return;
            }
        }

        if is_append {
            match crate::fs::fat::append_file_content(filename, &text_buf[..text_len]) {
                Ok(new_size) => {
                    vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                    vga::print_str("Successfully appended content to ");
                    vga::print_str(filename);
                    vga::print_str(" (New size: ");
                    vga::print_u64(new_size as u64);
                    vga::print_str(" bytes).\n");
                }
                Err(e) => {
                    vga::set_color(vga::Color::LightRed, vga::Color::Black);
                    vga::print_str("Error appending to file: ");
                    vga::print_str(e);
                    vga::print_str("\n");
                }
            }
        } else {
            match crate::fs::fat::write_file_content(filename, &text_buf[..text_len]) {
                Ok(_) => {
                    vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                    vga::print_str("Successfully wrote content to ");
                    vga::print_str(filename);
                    vga::print_str(".\n");
                }
                Err(e) => {
                    vga::set_color(vga::Color::LightRed, vga::Color::Black);
                    vga::print_str("Error writing to file: ");
                    vga::print_str(e);
                    vga::print_str("\n");
                }
            }
        }
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    }
}
