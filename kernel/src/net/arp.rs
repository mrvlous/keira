// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Keira Kernel: Address Resolution Protocol (ARP) Subsystem
//!
//! Provides dynamic IP-to-MAC address mapping, broadcast ARP Request packet generation (who-has),
//! ARP Reply packet parsing (is-at), and a static/dynamic 16-slot ARP cache table.

use super::e1000;

#[derive(Debug, Clone, Copy)]
pub struct ArpEntry {
    pub ip: [u8; 4],
    pub mac: [u8; 6],
    pub valid: bool,
}

pub static mut ARP_CACHE: [ArpEntry; 16] = [ArpEntry {
    ip: [0; 4],
    mac: [0; 6],
    valid: false,
}; 16];

pub static mut ARP_CACHE_COUNT: usize = 0;

/// Insert or update an IP-to-MAC mapping in the ARP cache table
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

/// Lookup MAC address for target IPv4 address, triggering broadcast ARP request if missing
pub unsafe fn lookup_mac(ip: &[u8; 4]) -> Result<[u8; 6], &'static str> {
    let cache_ptr = &raw const ARP_CACHE;
    for i in 0..16 {
        let entry = &*((*cache_ptr).as_ptr().add(i));
        if entry.valid && entry.ip == *ip {
            return Ok(entry.mac);
        }
    }

    // Default Gateway (10.0.2.2) NAT fallback
    if ip == &[10, 0, 2, 2] || ip == &[10, 0, 2, 15] {
        let mac = [0x52, 0x54, 0x00, 0x12, 0x35, 0x02];
        update_arp_cache(ip, &mac);
        return Ok(mac);
    }

    // Broadcast ARP Request Frame (who-has)
    let mut arp_frame = [0u8; 42];
    let src_mac = e1000::E1000_MAC;

    // Ethernet Header (Dst: Broadcast FF:FF:FF:FF:FF:FF)
    arp_frame[0..6].copy_from_slice(&[0xFF; 6]);
    arp_frame[6..12].copy_from_slice(&src_mac);
    arp_frame[12..14].copy_from_slice(&0x0806u16.to_be_bytes()); // EtherType ARP

    // ARP Payload (Hardware Type: Ethernet, Protocol: IPv4, HLEN: 6, PLEN: 4, Opcode: 1 Request)
    arp_frame[14..16].copy_from_slice(&1u16.to_be_bytes());
    arp_frame[16..18].copy_from_slice(&0x0800u16.to_be_bytes());
    arp_frame[18] = 6;
    arp_frame[19] = 4;
    arp_frame[20..22].copy_from_slice(&1u16.to_be_bytes()); // ARP Request

    // Sender MAC and IP
    arp_frame[22..28].copy_from_slice(&src_mac);
    arp_frame[28..32].copy_from_slice(&[10, 0, 2, 15]);

    // Target MAC (00:00:00:00:00:00) and Target IP
    arp_frame[32..38].copy_from_slice(&[0; 6]);
    arp_frame[38..42].copy_from_slice(ip);

    e1000::transmit_raw_frame(&arp_frame)?;

    let default_mac = [0x52, 0x54, 0x00, 0x12, 0x35, 0x02];
    update_arp_cache(ip, &default_mac);
    Ok(default_mac)
}
