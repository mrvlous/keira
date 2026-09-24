// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Unit tests for network device descriptors and constants.

use super::*;

#[test]
fn test_driver_constants_and_descriptors() {
    assert_eq!(rtl8139::RTL8139_VENDOR_ID, 0x10EC);
    assert_eq!(rtl8139::RTL8139_DEVICE_ID, 0x8139);

    assert_eq!(virtio::VIRTIO_NET_VENDOR_ID, 0x1AF4);
    assert_eq!(virtio::VIRTIO_NET_DEVICE_ID, 0x1000);

    let rtl = rtl8139::Rtl8139Device::new();
    assert!(!rtl.active);

    let virt = virtio::VirtioNetDevice::new();
    assert!(!virt.active);
}

#[test]
fn test_e1000_descriptor_sizes() {
    assert_eq!(core::mem::size_of::<e1000::E1000RxDesc>(), 16);
    assert_eq!(core::mem::size_of::<e1000::E1000TxDesc>(), 16);
}
