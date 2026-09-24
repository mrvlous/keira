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
