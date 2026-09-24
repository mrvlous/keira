// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Unit tests for DNS header, QNAME encoding, and localhost resolution.

use super::*;

#[test]
fn test_dns_header_size() {
    assert_eq!(core::mem::size_of::<DnsHeader>(), 12);
}

#[test]
fn test_dns_qname_encoding() {
    let mut buf = [0u8; 64];
    let len = encode_qname("keira.org", &mut buf).expect("Encoding failed");
    assert_eq!(&buf[..len], b"\x05keira\x03org\x00");
}

#[test]
fn test_resolve_localhost() {
    unsafe {
        let res = resolve_domain("localhost");
        assert_eq!(res, Ok([127, 0, 0, 1]));
    }
}
