// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Unit tests for Ethernet frame format and constants.

use super::*;

#[test]
fn test_ethertype_constants() {
    assert_eq!(ETHERTYPE_IPV4, 0x0800);
    assert_eq!(ETHERTYPE_ARP, 0x0806);
    assert_eq!(ETHERTYPE_IPV6, 0x86DD);
}

#[test]
fn test_ethernet_header_size() {
    assert_eq!(core::mem::size_of::<EthernetHeader>(), 14);
}
