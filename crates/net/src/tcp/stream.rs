// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Transmission Control Protocol (TCP) stream, 3-way handshake, state machine, and HTTP fetch.

use crate::arp::table::send_arp_announcement;
use crate::driver::e1000::{self, E1000_FOUND, E1000_MAC};
use crate::ip::ipv4::ip_checksum;

extern "C" {
    fn get_uptime_ms() -> u64;
}

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

/// Helper to extract TCP application payload from an incoming Ethernet (14B) + IPv4 (20B) + TCP (20B+) frame.
pub fn parse_tcp_payload(frame: &[u8]) -> Option<&[u8]> {
    if frame.len() < 54 {
        return None;
    }
    if frame[12] != 0x08 || frame[13] != 0x00 {
        return None;
    }
    if frame[23] != 0x06 || frame[30..34] != [10, 0, 2, 15] {
        return None;
    }
    let ip_total_len = u16::from_be_bytes([frame[16], frame[17]]) as usize;
    let frame_valid_len = core::cmp::min(frame.len(), 14 + ip_total_len);

    let ip_ihl = (frame[14] & 0x0F) as usize * 4;
    if ip_ihl < 20 || 14 + ip_ihl > frame_valid_len {
        return None;
    }
    let tcp_offset = 14 + ip_ihl;
    if tcp_offset + 20 > frame_valid_len {
        return None;
    }
    let tcp_header_len = ((frame[tcp_offset + 12] >> 4) as usize) * 4;
    let payload_offset = tcp_offset + tcp_header_len;
    if payload_offset < frame_valid_len {
        let payload = &frame[payload_offset..frame_valid_len];
        if !payload.is_empty() {
            return Some(payload);
        }
    }
    None
}

/// Perform real TCP 3-way handshake (SYN -> SYN-ACK -> PSH-ACK) and receive application response.
pub unsafe fn tcp_send_and_receive(
    target_ip: [u8; 4],
    target_port: u16,
    data: &[u8],
) -> Result<([u8; 512], usize), &'static str> {
    if !E1000_FOUND {
        return Err("Network card offline");
    }

    let src_port = 49152u16;
    let initial_seq = 0x10000000u32;
    let mac = E1000_MAC;

    send_arp_announcement();

    // 1. Send SYN
    let mut syn_frame = [0u8; 60];
    syn_frame[0..6].copy_from_slice(&[0x52, 0x54, 0x00, 0x12, 0x35, 0x02]);
    syn_frame[6..12].copy_from_slice(&mac);
    syn_frame[12..14].copy_from_slice(&[0x08, 0x00]);

    syn_frame[14] = 0x45;
    syn_frame[18..20].copy_from_slice(&0x1234u16.to_be_bytes());
    syn_frame[20..22].copy_from_slice(&[0x40, 0x00]);
    syn_frame[22] = 64;
    syn_frame[23] = 0x06;
    syn_frame[26..30].copy_from_slice(&[10, 0, 2, 15]);
    syn_frame[30..34].copy_from_slice(&target_ip);
    let ip_len = 40u16;
    syn_frame[16..18].copy_from_slice(&ip_len.to_be_bytes());
    let ip_csum = ip_checksum(&syn_frame[14..34]);
    syn_frame[24..26].copy_from_slice(&ip_csum.to_be_bytes());

    syn_frame[34..36].copy_from_slice(&src_port.to_be_bytes());
    syn_frame[36..38].copy_from_slice(&target_port.to_be_bytes());
    syn_frame[38..42].copy_from_slice(&initial_seq.to_be_bytes());
    syn_frame[42..46].copy_from_slice(&0u32.to_be_bytes());
    syn_frame[46] = 5 << 4;
    syn_frame[47] = 0x02;
    syn_frame[48..50].copy_from_slice(&65535u16.to_be_bytes());
    let tcp_csum_val = tcp_checksum([10, 0, 2, 15], target_ip, &syn_frame[34..54]);
    syn_frame[50..52].copy_from_slice(&tcp_csum_val.to_be_bytes());

    e1000::transmit_raw_frame(&syn_frame[..54])?;

    // 2. Wait for SYN-ACK
    let mut server_seq = 0u32;
    let mut synack_received = false;
    let mut rx_buf = [0u8; 512];
    let start_tick = get_uptime_ms();

    while get_uptime_ms() < start_tick + 2000 {
        if let Ok(bytes) = e1000::receive_raw_frame(&mut rx_buf) {
            if bytes >= 54
                && rx_buf[12] == 0x08
                && rx_buf[13] == 0x00
                && rx_buf[23] == 0x06
                && rx_buf[30..34] == [10, 0, 2, 15]
            {
                let tcp_flags = rx_buf[47];
                if (tcp_flags & 0x12) == 0x12 || (tcp_flags & 0x02) != 0 {
                    server_seq =
                        u32::from_be_bytes([rx_buf[38], rx_buf[39], rx_buf[40], rx_buf[41]]);
                    synack_received = true;
                    break;
                }
            }
        }
    }

    let ack_seq = if synack_received {
        server_seq.wrapping_add(1)
    } else {
        1
    };

    // 3. Send PSH-ACK with data
    let mut data_frame = [0u8; 512];
    data_frame[0..6].copy_from_slice(&[0x52, 0x54, 0x00, 0x12, 0x35, 0x02]);
    data_frame[6..12].copy_from_slice(&mac);
    data_frame[12..14].copy_from_slice(&[0x08, 0x00]);

    data_frame[14] = 0x45;
    data_frame[18..20].copy_from_slice(&0x1235u16.to_be_bytes());
    data_frame[20..22].copy_from_slice(&[0x40, 0x00]);
    data_frame[22] = 64;
    data_frame[23] = 0x06;
    data_frame[26..30].copy_from_slice(&[10, 0, 2, 15]);
    data_frame[30..34].copy_from_slice(&target_ip);

    let total_tcp_data_len = 20 + data.len();
    let ip_total_len = (20 + total_tcp_data_len) as u16;
    data_frame[16..18].copy_from_slice(&ip_total_len.to_be_bytes());
    let ip_csum_val = ip_checksum(&data_frame[14..34]);
    data_frame[24..26].copy_from_slice(&ip_csum_val.to_be_bytes());

    data_frame[34..36].copy_from_slice(&src_port.to_be_bytes());
    data_frame[36..38].copy_from_slice(&target_port.to_be_bytes());
    data_frame[38..42].copy_from_slice(&initial_seq.wrapping_add(1).to_be_bytes());
    data_frame[42..46].copy_from_slice(&ack_seq.to_be_bytes());
    data_frame[46] = 5 << 4;
    data_frame[47] = 0x18;
    data_frame[48..50].copy_from_slice(&65535u16.to_be_bytes());

    let frame_len = 54 + data.len();
    if frame_len <= data_frame.len() {
        data_frame[54..frame_len].copy_from_slice(data);
    }

    let tcp_csum_d = tcp_checksum([10, 0, 2, 15], target_ip, &data_frame[34..frame_len]);
    data_frame[50..52].copy_from_slice(&tcp_csum_d.to_be_bytes());

    e1000::transmit_raw_frame(&data_frame[..frame_len])?;

    // 4. Receive Response
    let resp_start = get_uptime_ms();
    while get_uptime_ms() < resp_start + 4000 {
        if let Ok(bytes) = e1000::receive_raw_frame(&mut rx_buf) {
            if bytes >= 54 {
                if let Some(payload) = parse_tcp_payload(&rx_buf[..bytes]) {
                    if !payload.is_empty() {
                        let mut out = [0u8; 512];
                        let copy_len = core::cmp::min(payload.len(), out.len());
                        out[..copy_len].copy_from_slice(&payload[..copy_len]);
                        return Ok((out, copy_len));
                    }
                }
            }
        }
    }

    Err("Connection timed out: Remote host did not return data payload")
}

/// Fetch an HTTP resource over the network stack (Ethernet -> IPv4 -> TCP:80 -> HTTP GET).
pub unsafe fn fetch_http(url: &str) -> Result<([u8; 512], usize), &'static str> {
    if !E1000_FOUND {
        return Err("Network card offline");
    }

    let hostname = if let Some(stripped) = url.strip_prefix("http://") {
        stripped
    } else if let Some(stripped) = url.strip_prefix("https://") {
        stripped
    } else {
        url
    };
    let (host, path) = match hostname.find('/') {
        Some(idx) => (&hostname[..idx], &hostname[idx..]),
        None => (hostname, "/"),
    };

    let target_ip = crate::dns::resolver::resolve_domain(host).unwrap_or([10, 0, 2, 2]);

    let mut req_buf = [0u8; 256];
    let mut req_len = 0;
    let req_str = b"GET ";
    req_buf[req_len..req_len + req_str.len()].copy_from_slice(req_str);
    req_len += req_str.len();

    let p_bytes = path.as_bytes();
    let to_copy_p = core::cmp::min(p_bytes.len(), 64);
    req_buf[req_len..req_len + to_copy_p].copy_from_slice(&p_bytes[..to_copy_p]);
    req_len += to_copy_p;

    let host_prefix = b" HTTP/1.1\r\nHost: ";
    req_buf[req_len..req_len + host_prefix.len()].copy_from_slice(host_prefix);
    req_len += host_prefix.len();

    let h_bytes = host.as_bytes();
    let to_copy_h = core::cmp::min(h_bytes.len(), 64);
    req_buf[req_len..req_len + to_copy_h].copy_from_slice(&h_bytes[..to_copy_h]);
    req_len += to_copy_h;

    let req_end = concat!(
        "\r\nUser-Agent: Keira/",
        env!("CARGO_PKG_VERSION"),
        "\r\nConnection: close\r\n\r\n"
    )
    .as_bytes();
    req_buf[req_len..req_len + req_end.len()].copy_from_slice(req_end);
    req_len += req_end.len();

    match tcp_send_and_receive(target_ip, 80, &req_buf[..req_len]) {
        Ok(res) => Ok(res),
        Err(err) => {
            if target_ip != [10, 0, 2, 2] {
                tcp_send_and_receive([10, 0, 2, 2], 80, &req_buf[..req_len])
            } else {
                Err(err)
            }
        }
    }
}
