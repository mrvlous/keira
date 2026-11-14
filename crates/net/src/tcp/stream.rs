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

//! Subsystem module: stream.

pub mod client {
    use crate::arp::protocol::{handle_arp_packet, send_arp_announcement};
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

    /// Retrieve and increment the global monotonic TCP sequence counter.
    pub fn get_next_client_seq() -> u32 {
        NEXT_CLIENT_SEQ.fetch_add(0x10000, Ordering::Relaxed)
    }

    /// Build standard Ethernet II + IPv4 + TCP frame into preallocated buffer.
    ///
    /// # Safety
    /// Reads static `E1000_MAC`.
    pub unsafe fn build_tcp_frame(
        target_ip: [u8; 4],
        target_port: u16,
        src_port: u16,
        seq: u32,
        ack: u32,
        flags: u8,
        data: &[u8],
        out: &mut [u8; 1518],
    ) -> usize {
        let mac = E1000_MAC;
        out[0..6].copy_from_slice(&[0x52, 0x54, 0x00, 0x12, 0x35, 0x02]);
        out[6..12].copy_from_slice(&mac);
        out[12..14].copy_from_slice(&[0x08, 0x00]);

        out[14] = 0x45;
        out[15] = 0x00;
        let ip_total_len = (40 + data.len()) as u16;
        out[16..18].copy_from_slice(&ip_total_len.to_be_bytes());
        out[18..20].copy_from_slice(&0x1234u16.to_be_bytes());
        out[20..22].copy_from_slice(&[0x40, 0x00]);
        out[22] = 64;
        out[23] = 0x06;
        out[24..26].copy_from_slice(&[0, 0]);
        out[26..30].copy_from_slice(&[10, 0, 2, 15]);
        out[30..34].copy_from_slice(&target_ip);
        let ip_csum_val = ip_checksum(&out[14..34]);
        out[24..26].copy_from_slice(&ip_csum_val.to_be_bytes());

        out[34..36].copy_from_slice(&src_port.to_be_bytes());
        out[36..38].copy_from_slice(&target_port.to_be_bytes());
        out[38..42].copy_from_slice(&seq.to_be_bytes());
        out[42..46].copy_from_slice(&ack.to_be_bytes());
        out[46] = 5 << 4;
        out[47] = flags;
        out[48..50].copy_from_slice(&65535u16.to_be_bytes());
        out[50..52].copy_from_slice(&[0, 0]);
        out[52..54].copy_from_slice(&[0, 0]);

        let frame_len = 54 + data.len();
        if !data.is_empty() && frame_len <= out.len() {
            out[54..frame_len].copy_from_slice(data);
        }

        let tcp_csum_val = tcp_checksum([10, 0, 2, 15], target_ip, &out[34..frame_len]);
        out[50..52].copy_from_slice(&tcp_csum_val.to_be_bytes());

        frame_len
    }

    /// Perform TCP 3-way handshake (SYN -> SYN-ACK -> ACK) and return server acknowledgement sequence.
    ///
    /// # Safety
    /// Transmits and receives raw Ethernet frames via e1000 DMA engine.
    pub unsafe fn tcp_connect(
        target_ip: [u8; 4],
        target_port: u16,
        src_port: u16,
        initial_seq: u32,
    ) -> Result<u32, &'static str> {
        if !E1000_FOUND {
            return Err("Network card offline");
        }

        send_arp_announcement();

        let mut frame = [0u8; 1518];
        let len = build_tcp_frame(
            target_ip,
            target_port,
            src_port,
            initial_seq,
            0,
            0x02,
            &[],
            &mut frame,
        );
        e1000::transmit_raw_frame(&frame[..len])?;

        let mut rx_buf = [0u8; 1518];
        let start_tick = get_uptime_ms();

        while get_uptime_ms() < start_tick + 3000 {
            if let Ok(bytes) = e1000::receive_raw_frame(&mut rx_buf) {
                if bytes >= 42 && rx_buf[12] == 0x08 && rx_buf[13] == 0x06 {
                    handle_arp_packet(&rx_buf[..bytes]);
                }
                if bytes >= 54
                    && rx_buf[12] == 0x08
                    && rx_buf[13] == 0x00
                    && rx_buf[23] == 0x06
                    && rx_buf[30..34] == [10, 0, 2, 15]
                    && rx_buf[36..38] == src_port.to_be_bytes()
                {
                    let tcp_flags = rx_buf[47];
                    if (tcp_flags & 0x04) != 0 {
                        return Err("Connection refused by peer");
                    }
                    if (tcp_flags & 0x12) == 0x12 || (tcp_flags & 0x02) != 0 {
                        let server_seq =
                            u32::from_be_bytes([rx_buf[38], rx_buf[39], rx_buf[40], rx_buf[41]]);
                        let ack_seq = server_seq.wrapping_add(1);

                        let ack_len = build_tcp_frame(
                            target_ip,
                            target_port,
                            src_port,
                            initial_seq.wrapping_add(1),
                            ack_seq,
                            0x10,
                            &[],
                            &mut frame,
                        );
                        let _ = e1000::transmit_raw_frame(&frame[..ack_len]);
                        return Ok(ack_seq);
                    }
                }
            }
        }

        Err("Connection timed out: SYN-ACK not received")
    }

    /// Transmit a TCP data segment with PSH | ACK flags.
    ///
    /// # Safety
    /// Transmits raw Ethernet frame via e1000 DMA engine.
    pub unsafe fn tcp_send_segment(
        target_ip: [u8; 4],
        target_port: u16,
        src_port: u16,
        seq: u32,
        ack: u32,
        data: &[u8],
    ) -> Result<(), &'static str> {
        if !E1000_FOUND {
            return Err("Network card offline");
        }

        let mut frame = [0u8; 1518];
        let len = build_tcp_frame(
            target_ip,
            target_port,
            src_port,
            seq,
            ack,
            0x18,
            data,
            &mut frame,
        );
        e1000::transmit_raw_frame(&frame[..len])
    }

    /// Transmit a standalone TCP acknowledgement frame.
    ///
    /// # Safety
    /// Transmits raw Ethernet frame via e1000 DMA engine.
    pub unsafe fn tcp_send_ack(
        target_ip: [u8; 4],
        target_port: u16,
        src_port: u16,
        seq: u32,
        ack: u32,
    ) -> Result<(), &'static str> {
        if !E1000_FOUND {
            return Err("Network card offline");
        }

        let mut frame = [0u8; 1518];
        let len = build_tcp_frame(
            target_ip,
            target_port,
            src_port,
            seq,
            ack,
            0x10,
            &[],
            &mut frame,
        );
        e1000::transmit_raw_frame(&frame[..len])
    }

    /// Transmit a TCP connection termination segment with FIN | ACK flags.
    ///
    /// # Safety
    /// Transmits raw Ethernet frame via e1000 DMA engine.
    pub unsafe fn tcp_send_fin(
        target_ip: [u8; 4],
        target_port: u16,
        src_port: u16,
        seq: u32,
        ack: u32,
    ) -> Result<(), &'static str> {
        if !E1000_FOUND {
            return Err("Network card offline");
        }

        let mut frame = [0u8; 1518];
        let len = build_tcp_frame(
            target_ip,
            target_port,
            src_port,
            seq,
            ack,
            0x11,
            &[],
            &mut frame,
        );
        e1000::transmit_raw_frame(&frame[..len])
    }

    /// Poll and receive an incoming TCP segment for the specified port pair.
    ///
    /// # Safety
    /// Receives raw Ethernet frame via e1000 DMA engine.
    pub unsafe fn tcp_receive_segment(
        _target_ip: [u8; 4],
        target_port: u16,
        src_port: u16,
        timeout_ms: u64,
        out_buf: &mut [u8],
    ) -> Result<(usize, u32, bool), &'static str> {
        if !E1000_FOUND {
            return Err("Network card offline");
        }

        let mut rx_buf = [0u8; 1518];
        let start_tick = get_uptime_ms();

        while get_uptime_ms() < start_tick + timeout_ms {
            if let Ok(bytes) = e1000::receive_raw_frame(&mut rx_buf) {
                if bytes >= 42 && rx_buf[12] == 0x08 && rx_buf[13] == 0x06 {
                    handle_arp_packet(&rx_buf[..bytes]);
                }
                if bytes >= 54
                    && rx_buf[12] == 0x08
                    && rx_buf[13] == 0x00
                    && rx_buf[23] == 0x06
                    && rx_buf[30..34] == [10, 0, 2, 15]
                    && rx_buf[36..38] == src_port.to_be_bytes()
                    && rx_buf[34..36] == target_port.to_be_bytes()
                {
                    let tcp_flags = rx_buf[47];
                    if (tcp_flags & 0x04) != 0 {
                        return Err("Connection reset by peer");
                    }
                    let is_fin = (tcp_flags & 0x01) != 0;
                    let server_seq =
                        u32::from_be_bytes([rx_buf[38], rx_buf[39], rx_buf[40], rx_buf[41]]);

                    if let Some(payload) = parse_tcp_payload(&rx_buf[..bytes]) {
                        if !payload.is_empty() {
                            let copy_len = core::cmp::min(payload.len(), out_buf.len());
                            out_buf[..copy_len].copy_from_slice(&payload[..copy_len]);
                            return Ok((copy_len, server_seq, is_fin));
                        }
                    }

                    if is_fin {
                        return Ok((0, server_seq, true));
                    }
                }
            }
        }

        Ok((0, 0, false))
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
        let src_port = get_next_src_port();
        let initial_seq = get_next_client_seq();
        let ack_seq = tcp_connect(target_ip, target_port, src_port, initial_seq)?;
        tcp_send_segment(
            target_ip,
            target_port,
            src_port,
            initial_seq.wrapping_add(1),
            ack_seq,
            data,
        )?;
        let mut rx = [0u8; 2048];
        let (received, _, _) =
            tcp_receive_segment(target_ip, target_port, src_port, 4000, &mut rx)?;
        if received > 0 {
            Ok((rx, received))
        } else {
            Err("Connection timed out: Remote host did not return data payload")
        }
    }
}

pub use self::client::*;
