// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Transmission Control Protocol (TCP) stream, 3-way handshake, continuous streaming, and HTTP fetch.

use crate::arp::table::send_arp_announcement;
use crate::driver::e1000::{self, E1000_FOUND, E1000_MAC};
use crate::ip::ipv4::ip_checksum;

use core::sync::atomic::{AtomicU16, Ordering};

extern "C" {
    fn get_uptime_ms() -> u64;
}

static NEXT_SRC_PORT: AtomicU16 = AtomicU16::new(49152);

/// Allocate next unique source port in range 49152..65000.
pub fn get_next_src_port() -> u16 {
    let port = NEXT_SRC_PORT.fetch_add(1, Ordering::Relaxed);
    if port < 49152 || port > 65000 {
        NEXT_SRC_PORT.store(49152, Ordering::Relaxed);
        49152
    } else {
        port
    }
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

/// Buffer capacity for streaming downloads (256 KB)
pub const STREAM_BUFFER_CAPACITY: usize = 262144;

/// Global static buffer for streaming download payloads (up to 256 KB)
pub static mut STREAM_DOWNLOAD_BUFFER: [u8; STREAM_BUFFER_CAPACITY] = [0u8; STREAM_BUFFER_CAPACITY];

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

/// Download a network resource using a full continuous TCP stream state machine.
/// Handles multi-packet reception, active sequence ACK replies, Content-Length tracking,
/// and live progress callback.
pub unsafe fn fetch_stream_download<F>(
    target_ip: [u8; 4],
    target_port: u16,
    request_data: &[u8],
    mut on_progress: F,
) -> Result<(&'static [u8], Option<usize>), &'static str>
where
    F: FnMut(usize, Option<usize>),
{
    if !E1000_FOUND {
        return Err("Network card offline");
    }

    let src_port = get_next_src_port();
    let initial_seq = 0x10000000u32.wrapping_add((get_uptime_ms() as u32).wrapping_mul(1103515245));
    let mac = E1000_MAC;

    // Drain any lingering stale packets from RX queue
    let mut drain_buf = [0u8; 2048];
    for _ in 0..16 {
        if e1000::receive_raw_frame(&mut drain_buf).is_err() {
            break;
        }
    }

    send_arp_announcement();

    // 1. Send SYN
    let mut syn_frame = [0u8; 60];
    syn_frame[0..6].copy_from_slice(&[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]);
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
    syn_frame[47] = 0x02; // SYN
    syn_frame[48..50].copy_from_slice(&65535u16.to_be_bytes());
    let tcp_csum_val = tcp_checksum([10, 0, 2, 15], target_ip, &syn_frame[34..54]);
    syn_frame[50..52].copy_from_slice(&tcp_csum_val.to_be_bytes());

    e1000::transmit_raw_frame(&syn_frame[..54])?;

    // 2. Wait for SYN-ACK with Retransmission (up to 2 attempts, 1000ms timeout per attempt)
    let mut server_seq = 0u32;
    let mut synack_received = false;
    let mut rx_buf = [0u8; 2048];

    for attempt in 0..2 {
        let start_tick = get_uptime_ms();
        if attempt > 0 {
            let _ = e1000::transmit_raw_frame(&syn_frame[..54]);
        }

        while get_uptime_ms() < start_tick + 1000 {
            if let Ok(bytes) = e1000::receive_raw_frame(&mut rx_buf) {
                if bytes >= 42 && rx_buf[12] == 0x08 && rx_buf[13] == 0x06 {
                    crate::arp::handle_arp_packet(&rx_buf[..bytes]);
                }
                if bytes >= 54
                    && rx_buf[12] == 0x08
                    && rx_buf[13] == 0x00
                    && rx_buf[23] == 0x06
                    && rx_buf[30..34] == [10, 0, 2, 15]
                    && (rx_buf[26..30] == target_ip || rx_buf[26..30] == [10, 0, 2, 2])
                {
                    let dst_p = u16::from_be_bytes([rx_buf[36], rx_buf[37]]);
                    if dst_p == src_port {
                        let tcp_flags = rx_buf[47];
                        if (tcp_flags & 0x12) == 0x12 || (tcp_flags & 0x02) != 0 {
                            server_seq = u32::from_be_bytes([
                                rx_buf[38], rx_buf[39], rx_buf[40], rx_buf[41],
                            ]);
                            synack_received = true;
                            break;
                        }
                    }
                }
            }
        }

        if synack_received {
            break;
        }
    }

    if !synack_received {
        return Err("Connection timed out: No SYN-ACK response from host");
    }

    let mut ack_seq = server_seq.wrapping_add(1);

    // 3. Send PSH-ACK with request data
    let mut data_frame = [0u8; 1024];
    data_frame[0..6].copy_from_slice(&[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]);
    data_frame[6..12].copy_from_slice(&mac);
    data_frame[12..14].copy_from_slice(&[0x08, 0x00]);

    data_frame[14] = 0x45;
    data_frame[18..20].copy_from_slice(&0x1235u16.to_be_bytes());
    data_frame[20..22].copy_from_slice(&[0x40, 0x00]);
    data_frame[22] = 64;
    data_frame[23] = 0x06;
    data_frame[26..30].copy_from_slice(&[10, 0, 2, 15]);
    data_frame[30..34].copy_from_slice(&target_ip);

    let total_tcp_data_len = 20 + request_data.len();
    let ip_total_len = (20 + total_tcp_data_len) as u16;
    data_frame[16..18].copy_from_slice(&ip_total_len.to_be_bytes());
    let ip_csum_val = ip_checksum(&data_frame[14..34]);
    data_frame[24..26].copy_from_slice(&ip_csum_val.to_be_bytes());

    data_frame[34..36].copy_from_slice(&src_port.to_be_bytes());
    data_frame[36..38].copy_from_slice(&target_port.to_be_bytes());
    data_frame[38..42].copy_from_slice(&initial_seq.wrapping_add(1).to_be_bytes());
    data_frame[42..46].copy_from_slice(&ack_seq.to_be_bytes());
    data_frame[46] = 5 << 4;
    data_frame[47] = 0x18; // PSH | ACK
    data_frame[48..50].copy_from_slice(&65535u16.to_be_bytes());

    let frame_len = 54 + request_data.len();
    if frame_len <= data_frame.len() {
        data_frame[54..frame_len].copy_from_slice(request_data);
    }

    let tcp_csum_d = tcp_checksum([10, 0, 2, 15], target_ip, &data_frame[34..frame_len]);
    data_frame[50..52].copy_from_slice(&tcp_csum_d.to_be_bytes());

    e1000::transmit_raw_frame(&data_frame[..frame_len])?;

    // Helper to send TCP ACK
    let send_ack = |client_seq: u32, ack_num: u32| -> Result<(), &'static str> {
        let mut ack_frame = [0u8; 54];
        ack_frame[0..6].copy_from_slice(&[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]);
        ack_frame[6..12].copy_from_slice(&mac);
        ack_frame[12..14].copy_from_slice(&[0x08, 0x00]);

        ack_frame[14] = 0x45;
        ack_frame[16..18].copy_from_slice(&40u16.to_be_bytes());
        ack_frame[18..20].copy_from_slice(&0x1236u16.to_be_bytes());
        ack_frame[20..22].copy_from_slice(&[0x40, 0x00]);
        ack_frame[22] = 64;
        ack_frame[23] = 0x06;
        ack_frame[26..30].copy_from_slice(&[10, 0, 2, 15]);
        ack_frame[30..34].copy_from_slice(&target_ip);
        let csum = ip_checksum(&ack_frame[14..34]);
        ack_frame[24..26].copy_from_slice(&csum.to_be_bytes());

        ack_frame[34..36].copy_from_slice(&src_port.to_be_bytes());
        ack_frame[36..38].copy_from_slice(&target_port.to_be_bytes());
        ack_frame[38..42].copy_from_slice(&client_seq.to_be_bytes());
        ack_frame[42..46].copy_from_slice(&ack_num.to_be_bytes());
        ack_frame[46] = 5 << 4;
        ack_frame[47] = 0x10; // ACK
        ack_frame[48..50].copy_from_slice(&65535u16.to_be_bytes());
        let tc = tcp_checksum([10, 0, 2, 15], target_ip, &ack_frame[34..54]);
        ack_frame[50..52].copy_from_slice(&tc.to_be_bytes());

        e1000::transmit_raw_frame(&ack_frame)
    };

    // 4. Continuous Streaming Receive Loop
    let mut total_downloaded = 0usize;
    let mut content_length: Option<usize> = None;
    let mut headers_stripped = false;
    let mut last_packet_time = get_uptime_ms();
    let mut data_retransmitted = false;
    let client_cur_seq = initial_seq.wrapping_add(1 + request_data.len() as u32);

    while get_uptime_ms() < last_packet_time + 15000 {
        if total_downloaded == 0 && !data_retransmitted && get_uptime_ms() > last_packet_time + 2500
        {
            data_retransmitted = true;
            let _ = e1000::transmit_raw_frame(&data_frame[..frame_len]);
        }

        if let Ok(bytes) = e1000::receive_raw_frame(&mut rx_buf) {
            if bytes >= 42 && rx_buf[12] == 0x08 && rx_buf[13] == 0x06 {
                crate::arp::handle_arp_packet(&rx_buf[..bytes]);
            }
            if bytes >= 54
                && rx_buf[12] == 0x08
                && rx_buf[13] == 0x00
                && rx_buf[23] == 0x06
                && (rx_buf[26..30] == target_ip || rx_buf[26..30] == [10, 0, 2, 2])
                && rx_buf[30..34] == [10, 0, 2, 15]
            {
                let dst_p = u16::from_be_bytes([rx_buf[36], rx_buf[37]]);
                if dst_p == src_port {
                    let tcp_flags = rx_buf[47];
                    let rx_seq =
                        u32::from_be_bytes([rx_buf[38], rx_buf[39], rx_buf[40], rx_buf[41]]);

                    if (tcp_flags & TCP_FLAG_RST) != 0 {
                        break;
                    }

                    if let Some(payload) = parse_tcp_payload(&rx_buf[..bytes]) {
                        if !payload.is_empty() {
                            last_packet_time = get_uptime_ms();

                            let data_to_append = if !headers_stripped {
                                let mut body_start = None;
                                for i in 0..payload.len().saturating_sub(3) {
                                    if &payload[i..i + 4] == b"\r\n\r\n" {
                                        body_start = Some(i + 4);
                                        if let Ok(hdr_str) = core::str::from_utf8(&payload[..i]) {
                                            for line in hdr_str.lines() {
                                                if let Some(cl_str) =
                                                    line.strip_prefix("Content-Length: ")
                                                {
                                                    if let Ok(cl) = cl_str.trim().parse::<usize>() {
                                                        content_length = Some(cl);
                                                    }
                                                } else if let Some(cl_str) =
                                                    line.strip_prefix("content-length: ")
                                                {
                                                    if let Ok(cl) = cl_str.trim().parse::<usize>() {
                                                        content_length = Some(cl);
                                                    }
                                                }
                                            }
                                        }
                                        break;
                                    }
                                }
                                headers_stripped = true;
                                if let Some(bs) = body_start {
                                    &payload[bs..]
                                } else {
                                    payload
                                }
                            } else {
                                payload
                            };

                            let available = STREAM_BUFFER_CAPACITY.saturating_sub(total_downloaded);
                            let copy_len = core::cmp::min(data_to_append.len(), available);
                            if copy_len > 0 {
                                let buf_ptr = (&raw mut STREAM_DOWNLOAD_BUFFER) as *mut u8;
                                core::ptr::copy_nonoverlapping(
                                    data_to_append.as_ptr(),
                                    buf_ptr.add(total_downloaded),
                                    copy_len,
                                );
                                total_downloaded += copy_len;
                                on_progress(total_downloaded, content_length);
                            }

                            // Send TCP ACK
                            ack_seq = rx_seq.wrapping_add(payload.len() as u32);
                            let _ = send_ack(client_cur_seq, ack_seq);

                            if let Some(cl) = content_length {
                                if total_downloaded >= cl {
                                    break;
                                }
                            }
                        }
                    }

                    if (tcp_flags & TCP_FLAG_FIN) != 0 {
                        let _ = send_ack(client_cur_seq, rx_seq.wrapping_add(1));
                        break;
                    }
                }
            }
        }
    }

    if total_downloaded == 0 {
        return Err("Connection timed out: Remote host did not return data payload");
    }

    let buf_slice = core::slice::from_raw_parts_mut(
        (&raw mut STREAM_DOWNLOAD_BUFFER) as *mut u8,
        total_downloaded,
    );
    let final_len = dechunk_in_place(buf_slice, total_downloaded);

    let out_slice =
        core::slice::from_raw_parts((&raw const STREAM_DOWNLOAD_BUFFER) as *const u8, final_len);

    Ok((out_slice, content_length))
}

/// In-place HTTP Transfer-Encoding chunked decoder.
pub fn dechunk_in_place(buf: &mut [u8], len: usize) -> usize {
    if len < 3 {
        return len;
    }

    // Check if the payload starts with a hex chunk header (e.g. "67\r\n{...")
    let mut is_chunked = false;
    let mut first_crlf = None;
    for i in 0..core::cmp::min(10, len.saturating_sub(1)) {
        if &buf[i..i + 2] == b"\r\n" {
            first_crlf = Some(i);
            break;
        }
    }

    if let Some(pos) = first_crlf {
        if pos > 0 {
            if let Ok(s) = core::str::from_utf8(&buf[..pos]) {
                if usize::from_str_radix(s.trim(), 16).is_ok() {
                    is_chunked = true;
                }
            }
        }
    }

    if !is_chunked {
        return len;
    }

    let mut read_idx = 0;
    let mut write_idx = 0;

    while read_idx < len {
        let mut crlf_pos = None;
        for i in read_idx..core::cmp::min(read_idx + 16, len.saturating_sub(1)) {
            if &buf[i..i + 2] == b"\r\n" {
                crlf_pos = Some(i);
                break;
            }
        }

        let header_end = match crlf_pos {
            Some(p) => p,
            None => break,
        };

        let hex_str = match core::str::from_utf8(&buf[read_idx..header_end]) {
            Ok(s) => s.trim(),
            Err(_) => break,
        };

        let chunk_size = match usize::from_str_radix(hex_str, 16) {
            Ok(sz) => sz,
            Err(_) => break,
        };

        if chunk_size == 0 {
            break;
        }

        read_idx = header_end + 2;
        let chunk_data_end = core::cmp::min(read_idx + chunk_size, len);
        let actual_chunk_len = chunk_data_end - read_idx;

        if actual_chunk_len > 0 {
            buf.copy_within(read_idx..chunk_data_end, write_idx);
            write_idx += actual_chunk_len;
        }

        read_idx = chunk_data_end;
        if read_idx + 2 <= len && &buf[read_idx..read_idx + 2] == b"\r\n" {
            read_idx += 2;
        }
    }

    if write_idx > 0 {
        write_idx
    } else {
        len
    }
}

/// Fetch an HTTP resource using continuous TCP streaming, returns downloaded payload slice and optional Content-Length.
pub unsafe fn fetch_http_stream<F>(
    url: &str,
    on_progress: F,
) -> Result<(&'static [u8], Option<usize>), &'static str>
where
    F: FnMut(usize, Option<usize>),
{
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

    let (host_str, target_port) = if let Some(colon) = host.find(':') {
        let h = &host[..colon];
        let p = host[colon + 1..].parse::<u16>().unwrap_or(80);
        (h, p)
    } else {
        (host, 80)
    };

    let target_ip = crate::dns::resolver::resolve_domain(host_str).unwrap_or([10, 0, 2, 2]);

    let mut req_buf = [0u8; 512];
    let mut req_len = 0;
    let req_str = b"GET ";
    req_buf[req_len..req_len + req_str.len()].copy_from_slice(req_str);
    req_len += req_str.len();

    let p_bytes = path.as_bytes();
    let to_copy_p = core::cmp::min(p_bytes.len(), 256);
    req_buf[req_len..req_len + to_copy_p].copy_from_slice(&p_bytes[..to_copy_p]);
    req_len += to_copy_p;

    let host_prefix = b" HTTP/1.1\r\nHost: ";
    req_buf[req_len..req_len + host_prefix.len()].copy_from_slice(host_prefix);
    req_len += host_prefix.len();

    let h_bytes = host.as_bytes();
    let to_copy_h = core::cmp::min(h_bytes.len(), 128);
    req_buf[req_len..req_len + to_copy_h].copy_from_slice(&h_bytes[..to_copy_h]);
    req_len += to_copy_h;

    let ua_prefix = b"\r\nUser-Agent: ";
    req_buf[req_len..req_len + ua_prefix.len()].copy_from_slice(ua_prefix);
    req_len += ua_prefix.len();

    let ua_bytes = crate::HTTP_USER_AGENT.as_bytes();
    req_buf[req_len..req_len + ua_bytes.len()].copy_from_slice(ua_bytes);
    req_len += ua_bytes.len();

    let req_end = b"\r\nConnection: close\r\n\r\n";
    req_buf[req_len..req_len + req_end.len()].copy_from_slice(req_end);
    req_len += req_end.len();

    match fetch_stream_download(target_ip, target_port, &req_buf[..req_len], on_progress) {
        Ok(res) => Ok(res),
        Err(err) => {
            if target_ip != [10, 0, 2, 2] {
                fetch_stream_download(
                    [10, 0, 2, 2],
                    target_port,
                    &req_buf[..req_len],
                    |_cur, _total| {},
                )
            } else {
                Err(err)
            }
        }
    }
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

    let (host_str, target_port) = if let Some(colon) = host.find(':') {
        let h = &host[..colon];
        let p = host[colon + 1..].parse::<u16>().unwrap_or(80);
        (h, p)
    } else {
        (host, 80)
    };

    let target_ip = crate::dns::resolver::resolve_domain(host_str).unwrap_or([10, 0, 2, 2]);

    let mut req_buf = [0u8; 512];
    let mut req_len = 0;
    let req_str = b"GET ";
    req_buf[req_len..req_len + req_str.len()].copy_from_slice(req_str);
    req_len += req_str.len();

    let p_bytes = path.as_bytes();
    let to_copy_p = core::cmp::min(p_bytes.len(), 128);
    req_buf[req_len..req_len + to_copy_p].copy_from_slice(&p_bytes[..to_copy_p]);
    req_len += to_copy_p;

    let host_prefix = b" HTTP/1.1\r\nHost: ";
    req_buf[req_len..req_len + host_prefix.len()].copy_from_slice(host_prefix);
    req_len += host_prefix.len();

    let h_bytes = host.as_bytes();
    let to_copy_h = core::cmp::min(h_bytes.len(), 128);
    req_buf[req_len..req_len + to_copy_h].copy_from_slice(&h_bytes[..to_copy_h]);
    req_len += to_copy_h;

    let ua_prefix = b"\r\nUser-Agent: ";
    req_buf[req_len..req_len + ua_prefix.len()].copy_from_slice(ua_prefix);
    req_len += ua_prefix.len();

    let ua_bytes = crate::HTTP_USER_AGENT.as_bytes();
    req_buf[req_len..req_len + ua_bytes.len()].copy_from_slice(ua_bytes);
    req_len += ua_bytes.len();

    let req_end = b"\r\nConnection: close\r\n\r\n";
    req_buf[req_len..req_len + req_end.len()].copy_from_slice(req_end);
    req_len += req_end.len();

    match tcp_send_and_receive(target_ip, target_port, &req_buf[..req_len]) {
        Ok(res) => Ok(res),
        Err(err) => {
            if target_ip != [10, 0, 2, 2] {
                tcp_send_and_receive([10, 0, 2, 2], target_port, &req_buf[..req_len])
            } else {
                Err(err)
            }
        }
    }
}
