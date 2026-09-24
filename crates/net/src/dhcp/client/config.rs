// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! DHCP lease configuration state and system defaults.

/// Network interface address lease configuration discovered via DHCP.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
