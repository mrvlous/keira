// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Unit tests for IPv4 header and address parsing.

use super::*;

#[test]
fn test_parse_ipv4_addr() {
    assert_eq!(parse_ipv4_addr("127.0.0.1"), Some([127, 0, 0, 1]));
    assert_eq!(parse_ipv4_addr("10.0.2.15"), Some([10, 0, 2, 15]));
    assert_eq!(
        parse_ipv4_addr("255.255.255.255"),
        Some([255, 255, 255, 255])
    );
    assert_eq!(parse_ipv4_addr("256.0.0.1"), None);
    assert_eq!(parse_ipv4_addr("10.0.1"), None);
    assert_eq!(parse_ipv4_addr("invalid"), None);
}

#[test]
fn test_ip_checksum_computation() {
    let header_bytes = [
        0x45, 0x00, 0x00, 0x3c, 0x1c, 0x46, 0x40, 0x00, 0x40, 0x06, 0x00, 0x00, 0xac, 0x10, 0x0a,
        0x63, 0xac, 0x10, 0x0a, 0x0c,
    ];
    let csum = ip_checksum(&header_bytes);
    assert_ne!(csum, 0);
}

#[test]
fn test_ipv4_routing_lookup() {
    unsafe {
        // 1. Loopback matches 127.0.0.0/8 on lo
        let route_lo = lookup_route(&[127, 0, 0, 1]);
        assert!(route_lo.is_some());
        assert_eq!(route_lo.unwrap().iface, "lo");

        // 2. Local subnet matches 10.0.2.0/24 on eth0
        let route_local = lookup_route(&[10, 0, 2, 15]);
        assert!(route_local.is_some());
        assert_eq!(route_local.unwrap().iface, "eth0");
        assert_eq!(route_local.unwrap().gateway, [0, 0, 0, 0]);

        // 3. Internet IP matches default gateway 0.0.0.0/0 via 10.0.2.2
        let route_inet = lookup_route(&[8, 8, 8, 8]);
        assert!(route_inet.is_some());
        assert_eq!(route_inet.unwrap().iface, "eth0");
        assert_eq!(route_inet.unwrap().gateway, [10, 0, 2, 2]);
    }
}

#[test]
fn test_ipv4_routing_table_modifications() {
    unsafe {
        // Add specific host route /32
        let res = add_route(
            [192, 168, 1, 50],
            [255, 255, 255, 255],
            [10, 0, 2, 1],
            "eth0",
            RTF_UP | RTF_HOST,
            10,
        );
        assert!(res.is_ok());

        // Verify longest prefix match chooses /32 over default
        let r = lookup_route(&[192, 168, 1, 50]);
        assert!(r.is_some());
        assert_eq!(r.unwrap().dest, [192, 168, 1, 50]);
        assert_eq!(r.unwrap().metric, 10);

        // Delete route
        assert!(del_route(&[192, 168, 1, 50], &[255, 255, 255, 255]).is_ok());

        // Now falls back to default gateway
        let r_fallback = lookup_route(&[192, 168, 1, 50]);
        assert!(r_fallback.is_some());
        assert_eq!(r_fallback.unwrap().dest, [0, 0, 0, 0]);
    }
}
