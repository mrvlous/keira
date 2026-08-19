// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//!
//! Provides virtual character device handlers for `/system/dev/null`, `/system/dev/zero`,
//! `/system/dev/random`, and `/system/dev/tty`, adhering to Keira's native filesystem hierarchy.

use crate::io::vga;

/// Read bytes from target character device node
pub unsafe fn read_dev_node(node_name: &str, buf: &mut [u8]) -> Result<usize, &'static str> {
    match node_name {
        "null" => {
            // Null device: Returns 0 bytes read (EOF)
            Ok(0)
        }
        "zero" => {
            // Zero device: Fills buffer with 0x00 bytes
            for slot in buf.iter_mut() {
                *slot = 0;
            }
            Ok(buf.len())
        }
        "random" => {
            // Random device: Fills buffer with pseudo-random bytes using CPU TSC tick
            let mut tsc: u64;
            core::arch::asm!("rdtsc", out("rax") tsc, out("rdx") _);
            for (idx, slot) in buf.iter_mut().enumerate() {
                let shift = (idx % 8) * 8;
                let byte = ((tsc >> shift) ^ (idx as u64 * 0x9E37_79B9)) as u8;
                *slot = byte;
            }
            Ok(buf.len())
        }
        "tty" => {
            // TTY device: Reads line from keyboard input
            Ok(0)
        }
        _ => Err("Unknown device node"),
    }
}

/// Write bytes to target character device node
pub unsafe fn write_dev_node(node_name: &str, buf: &[u8]) -> Result<usize, &'static str> {
    match node_name {
        "null" | "zero" | "random" => {
            // Null, Zero, Random: Discards written bytes successfully
            Ok(buf.len())
        }
        "tty" => {
            // TTY device: Prints buffer directly to VGA text console
            if let Ok(s) = core::str::from_utf8(buf) {
                vga::print_str(s);
            }
            Ok(buf.len())
        }
        _ => Err("Unknown device node"),
    }
}
