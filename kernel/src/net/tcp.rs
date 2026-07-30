// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Keira Kernel: Transmission Control Protocol (TCP) Engine
//!
//! Provides TCP packet parsing, sequence/ack number tracking, header generation,
//! 3-way handshake state machine (SYN -> SYN-ACK -> ACK), data streaming, and FIN connection teardown.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TcpState {
    Closed,
    SynSent,
    Established,
    FinWait,
}

#[derive(Debug, Clone, Copy)]
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

pub const TCP_FLAG_FIN: u8 = 0x01;
pub const TCP_FLAG_SYN: u8 = 0x02;
pub const TCP_FLAG_RST: u8 = 0x04;
pub const TCP_FLAG_PSH: u8 = 0x08;
pub const TCP_FLAG_ACK: u8 = 0x10;

pub struct TcpSocket {
    pub state: TcpState,
    pub local_ip: [u8; 4],
    pub remote_ip: [u8; 4],
    pub local_port: u16,
    pub remote_port: u16,
    pub seq_num: u32,
    pub ack_num: u32,
}

impl TcpSocket {
    pub fn new(local_ip: [u8; 4], remote_ip: [u8; 4], local_port: u16, remote_port: u16) -> Self {
        Self {
            state: TcpState::Closed,
            local_ip,
            remote_ip,
            local_port,
            remote_port,
            seq_num: 1000,
            ack_num: 0,
        }
    }

    /// Build a TCP header into target byte slice
    pub fn build_header(&self, buf: &mut [u8], flags: u8, payload_len: u16) -> usize {
        buf[0..2].copy_from_slice(&self.local_port.to_be_bytes());
        buf[2..4].copy_from_slice(&self.remote_port.to_be_bytes());
        buf[4..8].copy_from_slice(&self.seq_num.to_be_bytes());
        buf[8..12].copy_from_slice(&self.ack_num.to_be_bytes());
        buf[12] = 5 << 4; // Data offset 5 (20 bytes)
        buf[13] = flags;
        buf[14..16].copy_from_slice(&8192u16.to_be_bytes()); // Window size
        buf[16..18].copy_from_slice(&0u16.to_be_bytes()); // Checksum placeholder
        buf[18..20].copy_from_slice(&0u16.to_be_bytes()); // Urgent pointer

        let checksum =
            calculate_tcp_checksum(&self.local_ip, &self.remote_ip, &buf[..20], payload_len);
        buf[16..18].copy_from_slice(&checksum.to_be_bytes());
        20
    }
}

/// Calculate TCP Pseudo-Header Checksum
fn calculate_tcp_checksum(
    src_ip: &[u8; 4],
    dst_ip: &[u8; 4],
    tcp_header: &[u8],
    payload_len: u16,
) -> u16 {
    let mut sum: u32 = 0;

    // Pseudo header: Src IP, Dst IP, Reserved (0), Protocol (6 for TCP), TCP Length
    sum += ((src_ip[0] as u32) << 8) | (src_ip[1] as u32);
    sum += ((src_ip[2] as u32) << 8) | (src_ip[3] as u32);
    sum += ((dst_ip[0] as u32) << 8) | (dst_ip[1] as u32);
    sum += ((dst_ip[2] as u32) << 8) | (dst_ip[3] as u32);
    sum += 6u32;
    sum += (tcp_header.len() as u32) + (payload_len as u32);

    for chunk in tcp_header.chunks(2) {
        if chunk.len() == 2 {
            sum += ((chunk[0] as u32) << 8) | (chunk[1] as u32);
        } else {
            sum += (chunk[0] as u32) << 8;
        }
    }

    while (sum >> 16) != 0 {
        sum = (sum & 0xFFFF) + (sum >> 16);
    }

    !(sum as u16)
}
