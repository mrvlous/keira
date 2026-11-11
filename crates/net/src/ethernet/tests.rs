// SPDX-License-Identifier: GPL-2.0-only
/*
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra <mrvlous@proton.me>
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 */

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
