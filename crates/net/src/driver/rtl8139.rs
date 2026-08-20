// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Realtek RTL8139 Fast Ethernet PCI controller descriptor.

pub const RTL8139_VENDOR_ID: u16 = 0x10EC;
pub const RTL8139_DEVICE_ID: u16 = 0x8139;

/// Realtek RTL8139 device state descriptor.
#[derive(Clone, Copy, Debug)]
pub struct Rtl8139Device {
    pub io_base: u16,
    pub mac: [u8; 6],
    pub active: bool,
}

impl Rtl8139Device {
    pub const fn new() -> Self {
        Self {
            io_base: 0,
            mac: [0x52, 0x54, 0x00, 0x12, 0x34, 0x57],
            active: false,
        }
    }
}
