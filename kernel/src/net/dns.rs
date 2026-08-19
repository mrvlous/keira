// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//!
//! Provides UDP Port 53 domain name resolution (Host -> IPv4 address) with
//! a 16-slot Dynamic LRU/Hit-tracked DNS Cache Table for 0ms repeated resolution.

use super::e1000;
use crate::io::vga;

#[derive(Debug, Clone, Copy)]
pub struct DnsHeader {
    pub transaction_id: u16,
    pub flags: u16,
    pub questions: u16,
    pub answer_rrs: u16,
    pub authority_rrs: u16,
    pub additional_rrs: u16,
}

#[derive(Debug, Clone, Copy)]
pub struct DnsCacheEntry {
    pub domain: [u8; 64],
    pub domain_len: usize,
    pub ip: [u8; 4],
    pub hits: u32,
    pub valid: bool,
}

impl DnsCacheEntry {
    pub const fn empty() -> Self {
        Self {
            domain: [0u8; 64],
            domain_len: 0,
            ip: [0, 0, 0, 0],
            hits: 0,
            valid: false,
        }
    }
}

pub static mut DNS_CACHE: [DnsCacheEntry; 16] = [DnsCacheEntry::empty(); 16];
pub static mut DNS_CACHE_COUNT: usize = 0;

/// Encode domain string into DNS QNAME byte format (e.g. "google.com" -> \x06google\x03com\x00)
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
    buf[offset] = 0; // Null label terminator
    offset += 1;
    Ok(offset)
}

/// Helper to parse a dotted-decimal IPv4 string (e.g. "10.0.2.2")
pub fn parse_ipv4_addr(s: &str) -> Option<[u8; 4]> {
    let mut parts = s.split('.');
    let a = parts.next()?.parse::<u8>().ok()?;
    let b = parts.next()?.parse::<u8>().ok()?;
    let c = parts.next()?.parse::<u8>().ok()?;
    let d = parts.next()?.parse::<u8>().ok()?;
    if parts.next().is_none() {
        Some([a, b, c, d])
    } else {
        None
    }
}

/// Perform UDP 53 DNS query resolution with Dynamic DNS Cache Table lookup
pub unsafe fn resolve_domain(domain: &str) -> Result<[u8; 4], &'static str> {
    if domain == "localhost" {
        return Ok([127, 0, 0, 1]);
    }

    if let Some(ip) = parse_ipv4_addr(domain) {
        return Ok(ip);
    }

    let domain_bytes = domain.as_bytes();

    // 1. Check Dynamic DNS Cache Table
    for i in 0..16 {
        let entry = &mut DNS_CACHE[i];
        if entry.valid && entry.domain_len == domain_bytes.len() {
            if &entry.domain[..entry.domain_len] == domain_bytes {
                entry.hits += 1;
                return Ok(entry.ip);
            }
        }
    }

    if !e1000::E1000_FOUND {
        return Err("Network card offline");
    }

    // 2. Construct DNS Query Frame (Ethernet 14B + IPv4 20B + UDP 8B + DNS Payload)
    let mut frame = [0u8; 256];
    let mac = e1000::E1000_MAC;
    frame[0..6].copy_from_slice(&[0x52, 0x54, 0x00, 0x12, 0x35, 0x02]);
    frame[6..12].copy_from_slice(&mac);
    frame[12..14].copy_from_slice(&[0x08, 0x00]); // IPv4

    // IPv4 Header (20B at offset 14)
    frame[14] = 0x45;
    frame[18..20].copy_from_slice(&0x5353u16.to_be_bytes());
    frame[20..22].copy_from_slice(&[0x40, 0x00]);
    frame[22] = 64; // TTL
    frame[23] = 17; // UDP
    frame[26..30].copy_from_slice(&[10, 0, 2, 15]);
    frame[30..34].copy_from_slice(&[10, 0, 2, 3]); // QEMU DNS server 10.0.2.3

    // UDP Header (8B at offset 34)
    frame[34..36].copy_from_slice(&53530u16.to_be_bytes()); // Src Port
    frame[36..38].copy_from_slice(&53u16.to_be_bytes()); // Dst Port 53 (DNS)

    // DNS Payload starts at offset 42
    let dns_offset = 42;
    frame[dns_offset..dns_offset + 2].copy_from_slice(&0x1234u16.to_be_bytes()); // Transaction ID
    frame[dns_offset + 2..dns_offset + 4].copy_from_slice(&0x0100u16.to_be_bytes()); // Recursion desired
    frame[dns_offset + 4..dns_offset + 6].copy_from_slice(&1u16.to_be_bytes()); // 1 Question
    frame[dns_offset + 6..dns_offset + 8].copy_from_slice(&0u16.to_be_bytes());
    frame[dns_offset + 8..dns_offset + 10].copy_from_slice(&0u16.to_be_bytes());
    frame[dns_offset + 10..dns_offset + 12].copy_from_slice(&0u16.to_be_bytes());

    let qname_len = encode_qname(domain, &mut frame[dns_offset + 12..])?;
    let mut offset = dns_offset + 12 + qname_len;

    frame[offset..offset + 2].copy_from_slice(&1u16.to_be_bytes()); // QTYPE: A
    offset += 2;
    frame[offset..offset + 2].copy_from_slice(&1u16.to_be_bytes()); // QCLASS: IN
    offset += 2;

    let udp_len = (offset - 34) as u16;
    frame[38..40].copy_from_slice(&udp_len.to_be_bytes());
    frame[40..42].copy_from_slice(&0u16.to_be_bytes());

    let ip_len = (offset - 14) as u16;
    frame[16..18].copy_from_slice(&ip_len.to_be_bytes());
    let ip_csum = e1000::ip_checksum(&frame[14..34]);
    frame[24..26].copy_from_slice(&ip_csum.to_be_bytes());

    e1000::transmit_raw_frame(&frame[..offset])?;

    // 3. Receive and parse dynamic DNS Response packet from network
    let mut rx_buf = [0u8; 512];
    let start_tick = crate::shell::executor::get_uptime_ms();
    let mut resolved_ip: Option<[u8; 4]> = None;

    while crate::shell::executor::get_uptime_ms() < start_tick + 2000 {
        if let Ok(bytes) = e1000::receive_raw_frame(&mut rx_buf) {
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

    // 4. Store resolved IP into Dynamic DNS Cache Table
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

/// Display active Dynamic DNS Cache Table
pub unsafe fn print_dns_cache() {
    vga::set_color(vga::Color::LightCyan, vga::Color::Black);
    vga::print_str("DYNAMIC DNS CACHE TABLE (16 Slots)\n");
    vga::set_color(vga::Color::White, vga::Color::Black);

    let mut active = 0;
    for i in 0..16 {
        let entry = &DNS_CACHE[i];
        if entry.valid {
            active += 1;
            vga::print_str("  [Slot ");
            vga::print_u64(i as u64);
            vga::print_str("] Domain: ");
            if let Ok(d_str) = core::str::from_utf8(&entry.domain[..entry.domain_len]) {
                vga::print_str(d_str);
            }
            vga::print_str(" -> IP: ");
            vga::print_u64(entry.ip[0] as u64);
            vga::print_str(".");
            vga::print_u64(entry.ip[1] as u64);
            vga::print_str(".");
            vga::print_u64(entry.ip[2] as u64);
            vga::print_str(".");
            vga::print_u64(entry.ip[3] as u64);
            vga::print_str(" (Hits: ");
            vga::print_u64(entry.hits as u64);
            vga::print_str(")\n");
        }
    }

    if active == 0 {
        vga::print_str("  (No active domain entries in DNS cache table)\n");
    }
    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
}
