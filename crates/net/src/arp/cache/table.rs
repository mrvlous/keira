// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! In-kernel ARP cache table storage and lookup operations.

use super::entry::ArpEntry;
use keira_io::vga;

pub static mut ARP_CACHE: [ArpEntry; 16] = [ArpEntry {
    ip: [0; 4],
    mac: [0; 6],
    valid: false,
}; 16];

pub static mut ARP_CACHE_COUNT: usize = 0;

/// Insert or update an IP-to-MAC mapping in the ARP cache table.
///
/// # Safety
/// Directly mutates static `ARP_CACHE` table. Caller must ensure synchronized access.
pub unsafe fn update_arp_cache(ip: &[u8; 4], mac: &[u8; 6]) {
    let cache_ptr = &raw mut ARP_CACHE;
    for i in 0..16 {
        let entry = &mut *((*cache_ptr).as_mut_ptr().add(i));
        if entry.valid && entry.ip == *ip {
            entry.mac = *mac;
            return;
        }
    }

    let idx = ARP_CACHE_COUNT % 16;
    let entry = &mut *((*cache_ptr).as_mut_ptr().add(idx));
    *entry = ArpEntry {
        ip: *ip,
        mac: *mac,
        valid: true,
    };
    ARP_CACHE_COUNT += 1;
}

/// Display active ARP Cache and neighbor table.
///
/// # Safety
/// Prints directly to VGA console and inspects static `ARP_CACHE`.
pub unsafe fn print_arp_cache() {
    vga::set_color(vga::Color::White, vga::Color::Black);
    vga::print_str("ARP Cache & Neighbor Table:\n");
    vga::print_str("IP ADDRESS       HW TYPE     HW ADDRESS         INTERFACE\n");
    vga::print_str("---------------  ----------  -----------------  ---------\n");
    vga::set_color(vga::Color::LightGrey, vga::Color::Black);

    let cache_ptr = &raw const ARP_CACHE;
    let mut count = 0;
    for i in 0..16 {
        let entry = &*((*cache_ptr).as_ptr().add(i));
        if entry.valid {
            count += 1;
            let mut ip_str_len = 0;
            for (idx, octet) in entry.ip.iter().enumerate() {
                vga::print_u64(*octet as u64);
                let digits = if *octet >= 100 {
                    3
                } else if *octet >= 10 {
                    2
                } else {
                    1
                };
                ip_str_len += digits;
                if idx < 3 {
                    vga::print_str(".");
                    ip_str_len += 1;
                }
            }
            for _ in 0..(17usize.saturating_sub(ip_str_len)) {
                vga::print_str(" ");
            }
            vga::print_str("10/100/1G   ");
            for m_idx in 0..6 {
                let b = entry.mac[m_idx];
                let chars = b"0123456789ABCDEF";
                let buf = [chars[((b >> 4) & 0xF) as usize], chars[(b & 0xF) as usize]];
                if let Ok(s) = core::str::from_utf8(&buf) {
                    vga::print_str(s);
                }
                if m_idx < 5 {
                    vga::print_str(":");
                }
            }
            vga::print_str("  eth0\n");
        }
    }

    if count == 0 {
        vga::print_str("(No dynamic ARP neighbor entries resolved yet)\n");
    }
}

/// Formats live in-kernel ARP cache table in canonical /proc/net/arp format.
pub fn format_proc_arp(buf: &mut [u8]) -> usize {
    use core::fmt::Write;
    struct SliceWriter<'a> {
        b: &'a mut [u8],
        o: usize,
    }
    impl<'a> Write for SliceWriter<'a> {
        fn write_str(&mut self, s: &str) -> core::fmt::Result {
            let bytes = s.as_bytes();
            let rem = self.b.len().saturating_sub(self.o);
            let n = rem.min(bytes.len());
            self.b[self.o..self.o + n].copy_from_slice(&bytes[..n]);
            self.o += n;
            Ok(())
        }
    }

    let mut writer = SliceWriter { b: buf, o: 0 };
    let _ = writeln!(
        writer,
        "IP address       HW type     Flags       HW address            Mask     Device"
    );

    unsafe {
        let cache_ptr = &raw const ARP_CACHE;
        let mut count = 0;
        for i in 0..16 {
            let entry = &*(*cache_ptr).as_ptr().add(i);
            if !entry.valid {
                continue;
            }
            count += 1;
            let _ = writeln!(
                writer,
                "{}.{}.{}.{:<7} 0x1         0x2         {:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x} *        eth0",
                entry.ip[0],
                entry.ip[1],
                entry.ip[2],
                entry.ip[3],
                entry.mac[0],
                entry.mac[1],
                entry.mac[2],
                entry.mac[3],
                entry.mac[4],
                entry.mac[5]
            );
        }
        if count == 0 {
            // Default gateway mapping if ARP resolution has not run yet
            let _ = writeln!(
                writer,
                "{:<16} 0x{:<9x} 0x{:<9x} {:<21} {:<8} {:}",
                "10.0.2.2", 1, 2, "52:54:00:12:34:56", "*", "eth0"
            );
        }
    }

    writer.o
}
