// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Unit tests for TCP flags, port allocator, and chunked decoding.

use super::*;

#[test]
fn test_tcp_flags_constants() {
    assert_eq!(TCP_FLAG_FIN, 0x01);
    assert_eq!(TCP_FLAG_SYN, 0x02);
    assert_eq!(TCP_FLAG_RST, 0x04);
    assert_eq!(TCP_FLAG_PSH, 0x08);
    assert_eq!(TCP_FLAG_ACK, 0x10);
}

#[test]
fn test_get_next_src_port() {
    let p1 = get_next_src_port();
    let p2 = get_next_src_port();
    assert!(p1 >= 49152);
    assert_ne!(p1, p2);
}

#[test]
fn test_dechunk_in_place() {
    let mut data = *b"5\r\nhello\r\n0\r\n\r\n";
    let len = dechunk_in_place(&mut data, 15);
    assert_eq!(&data[..len], b"hello");
}
