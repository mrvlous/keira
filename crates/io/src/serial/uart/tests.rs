// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Unit tests for serial UART formatting routines.

use super::writer::{format_hex_buffer, format_u64_buffer};

#[test]
fn test_u64_formatting() {
    let mut buf = [0u8; 20];

    let start = format_u64_buffer(0, &mut buf);
    assert_eq!(&buf[start..], b"0");

    let start = format_u64_buffer(123456789, &mut buf);
    assert_eq!(&buf[start..], b"123456789");

    let start = format_u64_buffer(u64::MAX, &mut buf);
    assert_eq!(&buf[start..], b"18446744073709551615");
}

#[test]
fn test_hex_formatting() {
    let mut buf = [0u8; 16];

    let start = format_hex_buffer(0, &mut buf);
    assert_eq!(&buf[start..], b"0");

    let start = format_hex_buffer(0x1234_ABCD, &mut buf);
    assert_eq!(&buf[start..], b"1234ABCD");

    let start = format_hex_buffer(0xDEAD_BEEF_CAFE_BABE, &mut buf);
    assert_eq!(&buf[start..], b"DEADBEEFCAFEBABE");
}
