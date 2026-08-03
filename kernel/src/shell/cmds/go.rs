#![allow(unused_variables, unused_unsafe)]
// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Keira Kernel: Shell Command 'go'
//!
//! Implementation of the 'go' shell command.

use crate::io::vga;
use crate::shell::state::*;

pub fn run(parts: &mut core::str::SplitWhitespace) {
    unsafe {
        let arg = match parts.next() {
            Some("-h") | Some("--help") => {
                vga::print_str("Usage: go <path>\n\n");
                vga::print_str("Description:\n  Change current working directory on the active FAT16 volume. Supports relative (., ..) and absolute paths.\n\n");
                vga::print_str("Options:\n  -h, --help    Show this help message and exit\n\n");
                vga::print_str("Examples:\n  go /users/admin\n  go ..\n");
                return;
            }
            Some(s) => s,
            None => {
                vga::print_str("Usage: go <path>\n");
                return;
            }
        };
        unsafe {
            match crate::fs::fat::change_directory(arg) {
                Ok(_) => {
                    let mut temp_path = [0u8; 80];
                    let mut temp_len = 0;

                    if !arg.starts_with('/') {
                        temp_path[..SHELL_PATH_LEN].copy_from_slice(&SHELL_PATH[..SHELL_PATH_LEN]);
                        temp_len = SHELL_PATH_LEN;
                    }

                    for segment in arg.split('/') {
                        if segment.is_empty() || segment == "." {
                            continue;
                        }
                        if segment == ".." {
                            if temp_len > 0 {
                                let mut i = temp_len;
                                while i > 0 && temp_path[i - 1] != b'/' {
                                    i -= 1;
                                }
                                if i > 0 {
                                    temp_len = i - 1;
                                } else {
                                    temp_len = 0;
                                }
                            }
                        } else {
                            if temp_len > 0 {
                                if temp_len + 1 + segment.len() <= 80 {
                                    temp_path[temp_len] = b'/';
                                    temp_path[temp_len + 1..temp_len + 1 + segment.len()]
                                        .copy_from_slice(segment.as_bytes());
                                    temp_len += 1 + segment.len();
                                }
                            } else {
                                if segment.len() <= 80 {
                                    temp_path[..segment.len()].copy_from_slice(segment.as_bytes());
                                    temp_len = segment.len();
                                }
                            }
                        }
                    }

                    SHELL_PATH = [0u8; 80];
                    SHELL_PATH[..temp_len].copy_from_slice(&temp_path[..temp_len]);
                    SHELL_PATH_LEN = temp_len;
                }
                Err(e) => {
                    vga::print_str("go: ");
                    vga::print_str(e);
                    vga::print_str("\n");
                }
            }
        }
    }
}
