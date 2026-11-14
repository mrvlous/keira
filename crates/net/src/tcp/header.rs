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

//! Subsystem module: header.

pub mod checksum {
/// Compute standard TCP checksum with IPv4 pseudo-header.
pub fn tcp_checksum(src_ip: [u8; 4], dst_ip: [u8; 4], tcp_data: &[u8]) -> u16 {
    let mut sum: u32 = 0;
    sum = sum.wrapping_add(u16::from_be_bytes([src_ip[0], src_ip[1]]) as u32);
    sum = sum.wrapping_add(u16::from_be_bytes([src_ip[2], src_ip[3]]) as u32);
    sum = sum.wrapping_add(u16::from_be_bytes([dst_ip[0], dst_ip[1]]) as u32);
    sum = sum.wrapping_add(u16::from_be_bytes([dst_ip[2], dst_ip[3]]) as u32);
    sum = sum.wrapping_add(6u32);
    sum = sum.wrapping_add(tcp_data.len() as u32);

    let mut i = 0;
    while i + 1 < tcp_data.len() {
        let word = u16::from_be_bytes([tcp_data[i], tcp_data[i + 1]]);
        sum = sum.wrapping_add(word as u32);
        i += 2;
    }
    if i < tcp_data.len() {
        let word = u16::from_be_bytes([tcp_data[i], 0]);
        sum = sum.wrapping_add(word as u32);
    }
    while (sum >> 16) != 0 {
        sum = (sum & 0xFFFF) + (sum >> 16);
    }
    !(sum as u16)
}

}

pub mod types {
pub const TCP_FLAG_FIN: u8 = 0x01;
pub const TCP_FLAG_SYN: u8 = 0x02;
pub const TCP_FLAG_RST: u8 = 0x04;
pub const TCP_FLAG_PSH: u8 = 0x08;
pub const TCP_FLAG_ACK: u8 = 0x10;

/// Standard 20-byte TCP segment header.
#[repr(C, packed)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TcpHeader {
    pub src_port: u16,
    pub dst_port: u16,
    pub seq_num: u32,
    pub ack_num: u32,
    pub data_offset: u8,
    pub flags: u8,
    pub window_size: u16,
    pub checksum: u16,
    pub urgent_ptr: u16,
}

}

pub use self::checksum::*;
pub use self::types::*;
