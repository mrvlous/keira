// SPDX-License-Identifier: GPL-2.0-only

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
