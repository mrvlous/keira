// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! UDP 53 DNS client resolver and query transmission.

use crate::arp::protocol::{handle_arp_packet, send_arp_announcement};
use crate::dns::cache::{DNS_CACHE, DNS_CACHE_COUNT};
use crate::driver::e1000::{self, E1000_FOUND, E1000_MAC};
use crate::ip::ipv4::{ip_checksum, parse_ipv4_addr};

#[cfg(not(test))]
extern "C" {
    fn get_uptime_ms() -> u64;
}

#[cfg(test)]
unsafe fn get_uptime_ms() -> u64 {
    0
}

/// Encode domain string into DNS QNAME byte format (e.g. "google.com" -> \x06google\x03com\x00).
pub fn encode_qname(domain: &str, buf: &mut [u8]) -> Result<usize, &'static str> {
    let mut offset = 0usize;
    for label in domain.split('.') {
        if label.is_empty() {
            continue;
        }
        let len = label.len();
        if len > 63 || offset + len + 1 >= buf.len() {
            return Err("Domain label too long or buffer overflow");
        }
        buf[offset] = len as u8;
        offset += 1;
        buf[offset..offset + len].copy_from_slice(label.as_bytes());
        offset += len;
    }
    if offset >= buf.len() {
        return Err("Buffer overflow in QNAME encoding");
    }
    buf[offset] = 0;
    offset += 1;
    Ok(offset)
}

/// Perform UDP 53 DNS query resolution with Dynamic DNS Cache Table lookup.
///
/// # Safety
/// Transmits raw Ethernet frame and mutates static `DNS_CACHE` table.
pub unsafe fn resolve_domain(domain: &str) -> Result<[u8; 4], &'static str> {
    if domain == "localhost" {
        return Ok([127, 0, 0, 1]);
    }

    if let Some(ip) = parse_ipv4_addr(domain) {
        return Ok(ip);
    }

    let domain_bytes = domain.as_bytes();

    for i in 0..16 {
        let entry = &mut DNS_CACHE[i];
        if entry.valid
            && entry.domain_len == domain_bytes.len()
            && &entry.domain[..entry.domain_len] == domain_bytes
        {
            entry.hits += 1;
            return Ok(entry.ip);
        }
    }

    if !E1000_FOUND {
        return Err("Network card offline");
    }

    send_arp_announcement();

    let mut frame = [0u8; 256];
    let mac = E1000_MAC;
    frame[0..6].copy_from_slice(&[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]);
    frame[6..12].copy_from_slice(&mac);
    frame[12..14].copy_from_slice(&[0x08, 0x00]);

    frame[14] = 0x45;
    frame[18..20].copy_from_slice(&0x5353u16.to_be_bytes());
    frame[20..22].copy_from_slice(&[0x40, 0x00]);
    frame[22] = 64;
    frame[23] = 17;
    frame[26..30].copy_from_slice(&[10, 0, 2, 15]);
    frame[30..34].copy_from_slice(&[10, 0, 2, 3]);

    frame[34..36].copy_from_slice(&53530u16.to_be_bytes());
    frame[36..38].copy_from_slice(&53u16.to_be_bytes());

    let dns_offset = 42;
    frame[dns_offset..dns_offset + 2].copy_from_slice(&0x1234u16.to_be_bytes());
    frame[dns_offset + 2..dns_offset + 4].copy_from_slice(&0x0100u16.to_be_bytes());
    frame[dns_offset + 4..dns_offset + 6].copy_from_slice(&1u16.to_be_bytes());
    frame[dns_offset + 6..dns_offset + 8].copy_from_slice(&0u16.to_be_bytes());
    frame[dns_offset + 8..dns_offset + 10].copy_from_slice(&0u16.to_be_bytes());
    frame[dns_offset + 10..dns_offset + 12].copy_from_slice(&0u16.to_be_bytes());

    let qname_len = encode_qname(domain, &mut frame[dns_offset + 12..])?;
    let mut offset = dns_offset + 12 + qname_len;

    frame[offset..offset + 2].copy_from_slice(&1u16.to_be_bytes());
    offset += 2;
    frame[offset..offset + 2].copy_from_slice(&1u16.to_be_bytes());
    offset += 2;

    let udp_len = (offset - 34) as u16;
    frame[38..40].copy_from_slice(&udp_len.to_be_bytes());
    frame[40..42].copy_from_slice(&0u16.to_be_bytes());

    let ip_len = (offset - 14) as u16;
    frame[16..18].copy_from_slice(&ip_len.to_be_bytes());
    let ip_csum = ip_checksum(&frame[14..34]);
    frame[24..26].copy_from_slice(&ip_csum.to_be_bytes());

    e1000::transmit_raw_frame(&frame[..offset])?;

    let mut rx_buf = [0u8; 512];
    let start_tick = get_uptime_ms();
    let mut resolved_ip: Option<[u8; 4]> = None;

    while get_uptime_ms() < start_tick + 4000 {
        if let Ok(bytes) = e1000::receive_raw_frame(&mut rx_buf) {
            if bytes >= 42 && rx_buf[12] == 0x08 && rx_buf[13] == 0x06 {
                handle_arp_packet(&rx_buf[..bytes]);
            }
            if bytes >= 42 && rx_buf[12] == 0x08 && rx_buf[13] == 0x00 && rx_buf[23] == 17 {
                let dns_data = &rx_buf[42..bytes];
                if dns_data.len() >= 12 {
                    let tx_id = u16::from_be_bytes([dns_data[0], dns_data[1]]);
                    let ancount = u16::from_be_bytes([dns_data[6], dns_data[7]]);
                    if tx_id == 0x1234 && ancount > 0 {
                        let mut idx = 12;
                        while idx < dns_data.len() && dns_data[idx] != 0 {
                            if (dns_data[idx] & 0xC0) == 0xC0 {
                                idx += 2;
                                break;
                            }
                            idx += 1 + (dns_data[idx] as usize);
                        }
                        if idx < dns_data.len() && dns_data[idx] == 0 {
                            idx += 5;
                        }
                        while idx + 10 <= dns_data.len() {
                            if (dns_data[idx] & 0xC0) == 0xC0 {
                                idx += 2;
                            } else {
                                while idx < dns_data.len() && dns_data[idx] != 0 {
                                    idx += 1 + (dns_data[idx] as usize);
                                }
                                idx += 1;
                            }
                            if idx + 10 <= dns_data.len() {
                                let rtype = u16::from_be_bytes([dns_data[idx], dns_data[idx + 1]]);
                                let rdlen =
                                    u16::from_be_bytes([dns_data[idx + 8], dns_data[idx + 9]])
                                        as usize;
                                idx += 10;
                                if rtype == 1 && rdlen == 4 && idx + 4 <= dns_data.len() {
                                    resolved_ip = Some([
                                        dns_data[idx],
                                        dns_data[idx + 1],
                                        dns_data[idx + 2],
                                        dns_data[idx + 3],
                                    ]);
                                    break;
                                }
                                idx += rdlen;
                            }
                        }
                        break;
                    }
                }
            }
        }
    }

    let final_ip = match resolved_ip {
        Some(ip) => ip,
        None => return Err("DNS resolution failed: Domain not found or server timeout"),
    };

    let mut target_slot = 0;
    for i in 0..16 {
        if !DNS_CACHE[i].valid {
            target_slot = i;
            break;
        }
    }

    let len = core::cmp::min(domain_bytes.len(), 64);
    DNS_CACHE[target_slot].domain[..len].copy_from_slice(&domain_bytes[..len]);
    DNS_CACHE[target_slot].domain_len = len;
    DNS_CACHE[target_slot].ip = final_ip;
    DNS_CACHE[target_slot].hits = 1;
    DNS_CACHE[target_slot].valid = true;

    if target_slot >= DNS_CACHE_COUNT {
        DNS_CACHE_COUNT = target_slot + 1;
    }

    Ok(final_ip)
}
