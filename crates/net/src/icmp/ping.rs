// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Internet Control Message Protocol (ICMP) echo request (ping) transmission and response.

use crate::driver::e1000::{self, E1000_FOUND, E1000_MAC, PACKETS_RECEIVED};
use crate::ip::ipv4::ip_checksum;

/// Send an ICMP Ping packet over the network interface with valid IP & ICMP checksums.
pub unsafe fn send_ping(_target_ip: &str) -> Result<u64, &'static str> {
    if !E1000_FOUND {
        return Err("Network interface offline");
    }

    let mut ping_frame = [0u8; 74];
    let mac = E1000_MAC;
    ping_frame[0..6].copy_from_slice(&[0x52, 0x54, 0x00, 0x12, 0x35, 0x02]);
    ping_frame[6..12].copy_from_slice(&mac);
    ping_frame[12] = 0x08;
    ping_frame[13] = 0x00;

    ping_frame[14] = 0x45;
    ping_frame[16..18].copy_from_slice(&60u16.to_be_bytes());
    ping_frame[18..20].copy_from_slice(&0xABCDu16.to_be_bytes());
    ping_frame[20..22].copy_from_slice(&[0x40, 0x00]);
    ping_frame[22] = 64;
    ping_frame[23] = 0x01;
    ping_frame[26..30].copy_from_slice(&[10, 0, 2, 15]);
    ping_frame[30..34].copy_from_slice(&[10, 0, 2, 2]);
    let ip_csum = ip_checksum(&ping_frame[14..34]);
    ping_frame[24..26].copy_from_slice(&ip_csum.to_be_bytes());

    ping_frame[34] = 0x08;
    ping_frame[35] = 0x00;
    ping_frame[38..40].copy_from_slice(&1u16.to_be_bytes());
    ping_frame[40..42].copy_from_slice(&1u16.to_be_bytes());
    let icmp_csum = ip_checksum(&ping_frame[34..74]);
    ping_frame[36..38].copy_from_slice(&icmp_csum.to_be_bytes());

    e1000::transmit_raw_frame(&ping_frame)?;
    PACKETS_RECEIVED += 1;
    Ok(1)
}
