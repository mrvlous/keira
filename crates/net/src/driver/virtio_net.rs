// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! VirtIO-Net paravirtualized network interface descriptor.

pub const VIRTIO_NET_VENDOR_ID: u16 = 0x1AF4;
pub const VIRTIO_NET_DEVICE_ID: u16 = 0x1000;

/// VirtIO-Net device descriptor.
#[derive(Clone, Copy, Debug)]
pub struct VirtioNetDevice {
    pub io_base: u16,
    pub mac: [u8; 6],
    pub active: bool,
}

impl VirtioNetDevice {
    pub const fn new() -> Self {
        Self {
            io_base: 0,
            mac: [0x52, 0x54, 0x00, 0x12, 0x34, 0x58],
            active: false,
        }
    }
}
