// SPDX-License-Identifier: GPL-2.0-only
/*
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra <mrvlous@proton.me>
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 */

//! Character device drivers (/dev/null, /dev/zero, /dev/random, /dev/tty).

pub mod dispatcher {
    use super::{null, random, tty, zero};
    
    pub unsafe fn read_dev_node(node_name: &str, buf: &mut [u8]) -> Result<usize, &'static str> {
        match node_name {
            "null" => null::read(buf),
            "zero" => zero::read(buf),
            "random" | "urandom" => random::read(buf),
            "ptmx" => Ok(0),
            "tty" => tty::read(buf),
            _ => Err("Unknown device node"),
        }
    }
    
    pub unsafe fn write_dev_node(node_name: &str, buf: &[u8]) -> Result<usize, &'static str> {
        match node_name {
            "null" => null::write(buf),
            "zero" => zero::write(buf),
            "random" | "urandom" => random::write(buf),
            "ptmx" => Ok(buf.len()),
            "tty" => tty::write(buf),
            _ => Err("Unknown device node"),
        }
    }
    
    pub fn exists(node_name: &str) -> bool {
        let clean = node_name.trim_start_matches('/');
        matches!(
            clean,
            "null" | "zero" | "random" | "urandom" | "tty" | "ptmx"
        )
    }
    
    use crate::vfs::VirtualDirEntry;
    
    pub static DEV_ENTRIES: [VirtualDirEntry; 9] = [
        VirtualDirEntry {
            name: ".",
            d_type: 4,
            ino: 1,
        },
        VirtualDirEntry {
            name: "..",
            d_type: 4,
            ino: 1,
        },
        VirtualDirEntry {
            name: "null",
            d_type: 2,
            ino: 2,
        },
        VirtualDirEntry {
            name: "zero",
            d_type: 2,
            ino: 3,
        },
        VirtualDirEntry {
            name: "random",
            d_type: 2,
            ino: 4,
        },
        VirtualDirEntry {
            name: "urandom",
            d_type: 2,
            ino: 5,
        },
        VirtualDirEntry {
            name: "tty",
            d_type: 2,
            ino: 6,
        },
        VirtualDirEntry {
            name: "ptmx",
            d_type: 2,
            ino: 7,
        },
        VirtualDirEntry {
            name: "sda",
            d_type: 6,
            ino: 8,
        },
    ];
    
    pub fn get_dev_entry(index: usize) -> Option<VirtualDirEntry> {
        DEV_ENTRIES.get(index).copied()
    }
}

pub mod null {
    pub fn read(_buf: &mut [u8]) -> Result<usize, &'static str> {
        Ok(0)
    }
    
    pub fn write(buf: &[u8]) -> Result<usize, &'static str> {
        Ok(buf.len())
    }
}

pub mod random {
    pub fn read(buf: &mut [u8]) -> Result<usize, &'static str> {
        let mut i = 0;
        while i < buf.len() {
            let entropy = keira_arch::cpu::get_entropy_u32();
            let bytes = entropy.to_ne_bytes();
            let chunk = (buf.len() - i).min(4);
            buf[i..i + chunk].copy_from_slice(&bytes[..chunk]);
            i += chunk;
        }
        Ok(buf.len())
    }
    
    pub fn write(buf: &[u8]) -> Result<usize, &'static str> {
        Ok(buf.len())
    }
}

pub mod tty {
    use keira_io::vga;
    
    pub unsafe fn read(buf: &mut [u8]) -> Result<usize, &'static str> {
        let n = keira_io::tty::read_tty(buf);
        Ok(n)
    }
    
    pub fn write(buf: &[u8]) -> Result<usize, &'static str> {
        if let Ok(s) = core::str::from_utf8(buf) {
            vga::print_str(s);
        }
        Ok(buf.len())
    }
}

pub mod zero {
    pub fn read(buf: &mut [u8]) -> Result<usize, &'static str> {
        buf.fill(0);
        Ok(buf.len())
    }
    
    pub fn write(buf: &[u8]) -> Result<usize, &'static str> {
        Ok(buf.len())
    }
}

pub use self::dispatcher::{exists, get_dev_entry, read_dev_node, write_dev_node, DEV_ENTRIES};
