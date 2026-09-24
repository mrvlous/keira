// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! In-memory LRU DNS cache table and diagnostic display.

use keira_io::vga;

/// Dynamic DNS cache entry storing resolved domain and IPv4 address.
#[derive(Debug, Clone, Copy)]
pub struct DnsCacheEntry {
    pub domain: [u8; 64],
    pub domain_len: usize,
    pub ip: [u8; 4],
    pub hits: u32,
    pub valid: bool,
}

impl DnsCacheEntry {
    /// Construct an empty, invalid DNS cache entry.
    pub const fn empty() -> Self {
        Self {
            domain: [0u8; 64],
            domain_len: 0,
            ip: [0, 0, 0, 0],
            hits: 0,
            valid: false,
        }
    }
}

pub static mut DNS_CACHE: [DnsCacheEntry; 16] = [DnsCacheEntry::empty(); 16];
pub static mut DNS_CACHE_COUNT: usize = 0;

/// Display active Dynamic DNS Cache Table.
///
/// # Safety
/// Prints directly to VGA console and inspects static `DNS_CACHE`.
pub unsafe fn print_dns_cache() {
    vga::set_color(vga::Color::White, vga::Color::Black);
    vga::print_str("DYNAMIC DNS CACHE TABLE (16 Slots)\n");
    vga::set_color(vga::Color::White, vga::Color::Black);

    let mut active = 0;
    for i in 0..16 {
        let entry = &DNS_CACHE[i];
        if entry.valid {
            active += 1;
            vga::print_str("  [Slot ");
            vga::print_u64(i as u64);
            vga::print_str("] Domain: ");
            if let Ok(d_str) = core::str::from_utf8(&entry.domain[..entry.domain_len]) {
                vga::print_str(d_str);
            }
            vga::print_str(" -> IP: ");
            vga::print_u64(entry.ip[0] as u64);
            vga::print_str(".");
            vga::print_u64(entry.ip[1] as u64);
            vga::print_str(".");
            vga::print_u64(entry.ip[2] as u64);
            vga::print_str(".");
            vga::print_u64(entry.ip[3] as u64);
            vga::print_str(" (Hits: ");
            vga::print_u64(entry.hits as u64);
            vga::print_str(")\n");
        }
    }

    if active == 0 {
        vga::print_str("  (No active domain entries in DNS cache table)\n");
    }
    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
}
