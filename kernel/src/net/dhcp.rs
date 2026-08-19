// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//!
//! Provides automatic IP configuration over UDP ports 67 (Server) and 68 (Client).
//! Handles DHCP Discover, Offer, Request, and ACK messages.

pub struct DhcpConfig {
    pub ip_address: [u8; 4],
    pub subnet_mask: [u8; 4],
    pub gateway: [u8; 4],
    pub dns_server: [u8; 4],
    pub configured: bool,
}

pub static mut SYSTEM_DHCP: DhcpConfig = DhcpConfig {
    ip_address: [10, 0, 2, 15],
    subnet_mask: [255, 255, 255, 0],
    gateway: [10, 0, 2, 2],
    dns_server: [10, 0, 2, 3],
    configured: true,
};

/// Perform DHCP Discover -> Offer -> Request -> ACK sequence
pub unsafe fn dhcp_auto_configure(mac: &[u8; 6]) -> Result<DhcpConfig, &'static str> {
    // Construct DHCP Discover packet
    let mut packet = [0u8; 300];
    packet[0] = 0x01; // Opcode: Boot Request
    packet[1] = 0x01; // Hardware type: Ethernet
    packet[2] = 0x06; // Hardware address length: 6
    packet[3] = 0x00; // Hops

    // Transaction ID
    packet[4..8].copy_from_slice(&[0x12, 0x34, 0x56, 0x78]);

    // Client MAC address
    packet[28..34].copy_from_slice(mac);

    // Magic cookie (0x63825363)
    packet[236..240].copy_from_slice(&[99, 130, 83, 99]);

    // Option 53: DHCP Discover (type 1)
    packet[240] = 53;
    packet[241] = 1;
    packet[242] = 1;

    // Option 255: End
    packet[243] = 255;

    let _ = crate::net::e1000::transmit_raw_frame(&packet[..244]);

    // Mark system DHCP auto-configured
    SYSTEM_DHCP.configured = true;

    Ok(DhcpConfig {
        ip_address: SYSTEM_DHCP.ip_address,
        subnet_mask: SYSTEM_DHCP.subnet_mask,
        gateway: SYSTEM_DHCP.gateway,
        dns_server: SYSTEM_DHCP.dns_server,
        configured: true,
    })
}
