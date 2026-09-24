// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Unit tests for UDP packet header layout and checksum.

use super::*;

#[test]
fn test_udp_header_size() {
    assert_eq!(core::mem::size_of::<UdpHeader>(), 8);
}

#[test]
fn test_udp_checksum_calculation() {
    let src = [10, 0, 2, 15];
    let dst = [10, 0, 2, 2];
    let data = [0x12, 0x34, 0x56, 0x78];
    let csum = udp_checksum(src, dst, &data);
    assert_ne!(csum, 0);
}
