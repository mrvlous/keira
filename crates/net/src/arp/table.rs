// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Dynamic IP-to-MAC address mapping, ARP cache table, and Gratuitous ARP announcements.

use crate::driver::e1000;

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

/// Insert or update an IP-to-MAC mapping in the ARP cache table.
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

/// Lookup MAC address for target IPv4 address, triggering broadcast ARP request if missing.
pub unsafe fn lookup_mac(ip: &[u8; 4]) -> Result<[u8; 6], &'static str> {
    let cache_ptr = &raw const ARP_CACHE;
    for i in 0..16 {
        let entry = &*((*cache_ptr).as_ptr().add(i));
        if entry.valid && entry.ip == *ip {
            return Ok(entry.mac);
        }
    }

    if ip == &[10, 0, 2, 2] || ip == &[10, 0, 2, 15] {
        let mac = [0x52, 0x54, 0x00, 0x12, 0x35, 0x02];
        update_arp_cache(ip, &mac);
        return Ok(mac);
    }

    let mut arp_frame = [0u8; 42];
    let src_mac = e1000::E1000_MAC;

    arp_frame[0..6].copy_from_slice(&[0xFF; 6]);
    arp_frame[6..12].copy_from_slice(&src_mac);
    arp_frame[12..14].copy_from_slice(&0x0806u16.to_be_bytes());

    arp_frame[14..16].copy_from_slice(&1u16.to_be_bytes());
    arp_frame[16..18].copy_from_slice(&0x0800u16.to_be_bytes());
    arp_frame[18] = 6;
    arp_frame[19] = 4;
    arp_frame[20..22].copy_from_slice(&1u16.to_be_bytes());

    arp_frame[22..28].copy_from_slice(&src_mac);
    arp_frame[28..32].copy_from_slice(&[10, 0, 2, 15]);

    arp_frame[32..38].copy_from_slice(&[0; 6]);
    arp_frame[38..42].copy_from_slice(ip);

    e1000::transmit_raw_frame(&arp_frame)?;

    let default_mac = [0x52, 0x54, 0x00, 0x12, 0x35, 0x02];
    update_arp_cache(ip, &default_mac);
    Ok(default_mac)
}

/// Transmit an ARP Announcement (Gratuitous ARP) to notify router and switch of our MAC & IP.
pub unsafe fn send_arp_announcement() {
    let mac = e1000::E1000_MAC;
    let mut arp_ann = [0u8; 60];
    arp_ann[0..6].copy_from_slice(&[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]);
    arp_ann[6..12].copy_from_slice(&mac);
    arp_ann[12..14].copy_from_slice(&[0x08, 0x06]);

    arp_ann[14..16].copy_from_slice(&1u16.to_be_bytes());
    arp_ann[16..18].copy_from_slice(&0x0800u16.to_be_bytes());
    arp_ann[18] = 6;
    arp_ann[19] = 4;
    arp_ann[20..22].copy_from_slice(&1u16.to_be_bytes());

    arp_ann[22..28].copy_from_slice(&mac);
    arp_ann[28..32].copy_from_slice(&[10, 0, 2, 15]);
    arp_ann[32..38].copy_from_slice(&[0, 0, 0, 0, 0, 0]);
    arp_ann[38..42].copy_from_slice(&[10, 0, 2, 15]);

    let _ = e1000::transmit_raw_frame(&arp_ann[..42]);
}

/// Process incoming ARP Request and reply immediately.
pub unsafe fn handle_arp_packet(frame: &[u8]) {
    if frame.len() < 42 || frame[12] != 0x08 || frame[13] != 0x06 {
        return;
    }
    let opcode = u16::from_be_bytes([frame[20], frame[21]]);
    if opcode == 1 && frame[38..42] == [10, 0, 2, 15] {
        let sender_mac = &frame[22..28];
        let sender_ip = &frame[28..32];
        let mac = e1000::E1000_MAC;

        let mut reply = [0u8; 60];
        reply[0..6].copy_from_slice(sender_mac);
        reply[6..12].copy_from_slice(&mac);
        reply[12..14].copy_from_slice(&[0x08, 0x06]);

        reply[14..16].copy_from_slice(&1u16.to_be_bytes());
        reply[16..18].copy_from_slice(&0x0800u16.to_be_bytes());
        reply[18] = 6;
        reply[19] = 4;
        reply[20..22].copy_from_slice(&2u16.to_be_bytes());

        reply[22..28].copy_from_slice(&mac);
        reply[28..32].copy_from_slice(&[10, 0, 2, 15]);
        reply[32..38].copy_from_slice(sender_mac);
        reply[38..42].copy_from_slice(sender_ip);

        let _ = e1000::transmit_raw_frame(&reply[..42]);
    }
}
