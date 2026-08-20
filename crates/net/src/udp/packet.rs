// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! User Datagram Protocol (UDP) packet header construction and checksum computation.

use crate::ip::ipv4::ip_checksum;

/// 8-byte UDP packet header.
#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
pub struct UdpHeader {
    pub src_port: u16,
    pub dst_port: u16,
    pub length: u16,
    pub checksum: u16,
}

/// Compute UDP checksum with IPv4 pseudo-header.
pub fn udp_checksum(src_ip: [u8; 4], dst_ip: [u8; 4], udp_data: &[u8]) -> u16 {
    let mut sum: u32 = 0;
    sum = sum.wrapping_add(u16::from_be_bytes([src_ip[0], src_ip[1]]) as u32);
    sum = sum.wrapping_add(u16::from_be_bytes([src_ip[2], src_ip[3]]) as u32);
    sum = sum.wrapping_add(u16::from_be_bytes([dst_ip[0], dst_ip[1]]) as u32);
    sum = sum.wrapping_add(u16::from_be_bytes([dst_ip[2], dst_ip[3]]) as u32);
    sum = sum.wrapping_add(17u32);
    sum = sum.wrapping_add(udp_data.len() as u32);

    let mut i = 0;
    while i + 1 < udp_data.len() {
        let word = u16::from_be_bytes([udp_data[i], udp_data[i + 1]]);
        sum = sum.wrapping_add(word as u32);
        i += 2;
    }
    if i < udp_data.len() {
        let word = u16::from_be_bytes([udp_data[i], 0]);
        sum = sum.wrapping_add(word as u32);
    }
    while (sum >> 16) != 0 {
        sum = (sum & 0xFFFF) + (sum >> 16);
    }
    !(sum as u16)
}
