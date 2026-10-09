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

#[test]
fn test_loopback_constants_and_address_predicates() {
    assert_eq!(LOOPBACK_NAME, "lo");
    assert_eq!(LOOPBACK_IP, [127, 0, 0, 1]);
    assert_eq!(LOOPBACK_NETMASK, [255, 0, 0, 0]);
    assert_eq!(LOOPBACK_MAC, [0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
    assert_eq!(LOOPBACK_MTU, 65536);

    assert!(is_loopback_addr(&[127, 0, 0, 1]));
    assert!(is_loopback_addr(&[127, 10, 20, 30]));
    assert!(!is_loopback_addr(&[10, 0, 2, 15]));
    assert!(!is_loopback_addr(&[192, 168, 1, 1]));

    assert!(is_loopback_str("127.0.0.1"));
    assert!(is_loopback_str("127.0.1.1"));
    assert!(is_loopback_str("localhost"));
    assert!(!is_loopback_str("8.8.8.8"));
    assert!(!is_loopback_str("keira-os.org"));
}

#[test]
fn test_loopback_packet_transmission_and_metrics() {
    unsafe {
        reset_loopback_stats();
        let (tx0, rx0, bytes0) = get_loopback_stats();
        assert_eq!(tx0, 0);
        assert_eq!(rx0, 0);
        assert_eq!(bytes0, 0);

        let payload = b"Hello Keira Loopback!";
        let res = transmit_loopback_packet(payload);
        assert_eq!(res, Ok(payload.len()));

        let (tx1, rx1, bytes1) = get_loopback_stats();
        assert_eq!(tx1, 1);
        assert_eq!(rx1, 1);
        assert_eq!(bytes1, payload.len() as u64);

        let ping_res = send_loopback_ping();
        assert_eq!(ping_res, Ok(1));

        let (tx2, rx2, bytes2) = get_loopback_stats();
        assert_eq!(tx2, 2);
        assert_eq!(rx2, 2);
        assert_eq!(bytes2, (payload.len() + 64) as u64);

        reset_loopback_stats();
    }
}
