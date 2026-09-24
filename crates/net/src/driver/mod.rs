// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Network interface device drivers (Intel e1000, Realtek RTL8139, VirtIO-Net).

pub mod e1000;
pub mod rtl8139;
pub mod virtio;

pub use virtio as virtio_net;

#[cfg(test)]
mod tests;

pub use e1000::{
    init as e1000_init, receive_raw_frame, transmit_raw_frame, E1000RxDesc, E1000TxDesc,
    E1000_FOUND, E1000_IO_BASE, E1000_MAC, E1000_MEM_BASE, PACKETS_RECEIVED, PACKETS_SENT,
};
pub use rtl8139::{Rtl8139Device, RTL8139_DEVICE_ID, RTL8139_VENDOR_ID};
pub use virtio::{VirtioNetDevice, VIRTIO_NET_DEVICE_ID, VIRTIO_NET_VENDOR_ID};
