#![allow(unused_variables, unused_unsafe)]
// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Keira Kernel: Shell Command 'script'
//!
//! Implementation of the 'script' shell command.

static mut SCRIPT_BUFFER: [u8; 65536] = [0; 65536];

use crate::io::vga;
use crate::shell::executor::*;

pub fn run(parts: &mut core::str::SplitWhitespace) {
    let arg = match parts.next() {
        Some("-h") | Some("--help") => {
            vga::print_str("Usage: script <filename.sh>\n\n");
            vga::print_str("Description:\n  Read and execute terminal shell commands sequentially line-by-line from a script file on FAT16 storage.\n\n");
            vga::print_str("Options:\n  -h, --help    Show this help message and exit\n\n");
            vga::print_str("Examples:\n  script build.sh\n");
            return;
        }
        Some(s) => s,
        None => {
            vga::print_str("Usage: script <filename.sh>\n");
            return;
        }
    };
    unsafe {
        let script_buf = &mut *core::ptr::addr_of_mut!(SCRIPT_BUFFER);
        let read_res = crate::fs::vfs::read_file(arg, script_buf);
        match read_res {
            Ok(len) => {
                let content = &script_buf[..len];
                let mut line_start = 0;
                for i in 0..=len {
                    if i == len || content[i] == b'\n' || content[i] == b'\r' {
                        if i > line_start {
                            let line_bytes = &content[line_start..i];
                            let mut start = 0;
                            let mut end = line_bytes.len();
                            while start < end
                                && (line_bytes[start] == b' '
                                    || line_bytes[start] == b'\t'
                                    || line_bytes[start] == b'\r')
                            {
                                start += 1;
                            }
                            while end > start
                                && (line_bytes[end - 1] == b' '
                                    || line_bytes[end - 1] == b'\t'
                                    || line_bytes[end - 1] == b'\r')
                            {
                                end -= 1;
                            }
                            let trimmed = &line_bytes[start..end];
                            if !trimmed.is_empty() {
                                if let Ok(cmd_str) = core::str::from_utf8(trimmed) {
                                    vga::print_str("Executing: ");
                                    vga::print_str(cmd_str);
                                    vga::print_str("\n");
                                    execute_command_inner(cmd_str);
                                }
                            }
                        }
                        line_start = i + 1;
                    }
                }
            }
            Err(e) => {
                vga::print_str("script: ");
                vga::print_str(e);
                vga::print_str("\n");
            }
        }
    }
}
