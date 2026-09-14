// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Virtual character device nodes (`/system/dev/null`, `/system/dev/zero`, `/system/dev/random`, `/system/dev/tty`).

use keira_io::vga;

/// Read bytes from a special character device node.
pub unsafe fn read_dev_node(node_name: &str, buf: &mut [u8]) -> Result<usize, &'static str> {
    match node_name {
        "null" => Ok(0),
        "zero" => {
            buf.fill(0);
            Ok(buf.len())
        }
        "random" => {
            let lo: u32;
            let hi: u32;
            core::arch::asm!("rdtsc", out("eax") lo, out("edx") hi);
            let tsc = ((hi as u64) << 32) | (lo as u64);
            for (idx, slot) in buf.iter_mut().enumerate() {
                let shift = (idx % 8) * 8;
                let byte = ((tsc >> shift) ^ (idx as u64 * 0x9E37_79B9)) as u8;
                *slot = byte;
            }
            Ok(buf.len())
        }
        "tty" => {
            let mut read_bytes = 0;
            while read_bytes < buf.len() {
                if let Some(c) = keira_io::ps2::pop_input_char() {
                    buf[read_bytes] = c;
                    read_bytes += 1;
                } else {
                    break;
                }
            }
            Ok(read_bytes)
        }
        _ => Err("Unknown device node"),
    }
}

/// Write bytes to a special character device node.
pub unsafe fn write_dev_node(node_name: &str, buf: &[u8]) -> Result<usize, &'static str> {
    match node_name {
        "null" | "zero" | "random" => Ok(buf.len()),
        "tty" => {
            if let Ok(s) = core::str::from_utf8(buf) {
                vga::print_str(s);
            }
            Ok(buf.len())
        }
        _ => Err("Unknown device node"),
    }
}
