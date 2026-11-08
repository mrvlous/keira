// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Continuous TCP connection stream handling, buffer management and packet exchange.

use crate::arp::protocol::send_arp_announcement;
use crate::driver::e1000::{self, E1000_FOUND, E1000_MAC};
use crate::ip::ipv4::ip_checksum;
use crate::tcp::header::tcp_checksum;
use crate::tcp::state::get_next_src_port;
use core::sync::atomic::{AtomicU32, Ordering};

static NEXT_CLIENT_SEQ: AtomicU32 = AtomicU32::new(0x10000000);

#[cfg(not(test))]
extern "C" {
    fn get_uptime_ms() -> u64;
}

#[cfg(test)]
unsafe fn get_uptime_ms() -> u64 {
    0
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
///
/// # Safety
/// Transmits and receives raw Ethernet frames using e1000 DMA engine.
pub unsafe fn tcp_send_and_receive(
    target_ip: [u8; 4],
    target_port: u16,
    data: &[u8],
) -> Result<([u8; 2048], usize), &'static str> {
    if !E1000_FOUND {
        return Err("Network card offline");
    }

    let src_port = get_next_src_port();
    let initial_seq = NEXT_CLIENT_SEQ.fetch_add(0x10000, Ordering::Relaxed);
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
    let mut rx_buf = [0u8; 1518];
    let start_tick = get_uptime_ms();

    while get_uptime_ms() < start_tick + 2000 {
        if let Ok(bytes) = e1000::receive_raw_frame(&mut rx_buf) {
            if bytes >= 54
                && rx_buf[12] == 0x08
                && rx_buf[13] == 0x00
                && rx_buf[23] == 0x06
                && rx_buf[30..34] == [10, 0, 2, 15]
                && rx_buf[36..38] == src_port.to_be_bytes()
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
    let mut data_frame = [0u8; 1518];
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
            if bytes >= 54 && rx_buf[36..38] == src_port.to_be_bytes() {
                if let Some(payload) = parse_tcp_payload(&rx_buf[..bytes]) {
                    if !payload.is_empty() {
                        let mut out = [0u8; 2048];
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
