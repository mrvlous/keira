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
//! Implementation of the 'wait' shell command.

use crate::executor::*;
use keira_io::vga;

pub fn run(parts: &mut core::str::SplitWhitespace) {
    unsafe {
        let arg = match parts.next() {
            Some("-h") | Some("--help") => {
                vga::print_str("Usage: wait <milliseconds>\n\n");
                vga::print_str("Description:\n  Suspend shell execution for specified duration in milliseconds.\n\n");
                vga::print_str("Options:\n  -h, --help    Show this help message and exit\n\n");
                vga::print_str("Examples:\n  wait 1000\n");
                return;
            }
            Some(s) => s,
            None => {
                vga::print_str("Usage: wait <milliseconds>\n");
                return;
            }
        };
        let mut ms = 0u64;
        for &b in arg.as_bytes() {
            if (b'0'..=b'9').contains(&b) {
                ms = (ms * 10) + (b - b'0') as u64;
            } else {
                vga::print_str("Error: Invalid number.\n");
                return;
            }
        }
        let start = unsafe { get_uptime_ms() };
        while unsafe { get_uptime_ms() } < start + ms {
            unsafe {
                core::arch::asm!("hlt");
            }
        }
    }
}
