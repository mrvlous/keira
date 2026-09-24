// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Unit tests for ARP cache manipulation and entry validation.

use super::*;

#[test]
fn test_arp_cache_insertion() {
    unsafe {
        let ip = [192, 168, 1, 100];
        let mac = [0x00, 0x11, 0x22, 0x33, 0x44, 0x55];

        update_arp_cache(&ip, &mac);

        let cached = lookup_mac(&ip);
        assert_eq!(cached, Ok(mac));
    }
}
