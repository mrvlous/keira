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
