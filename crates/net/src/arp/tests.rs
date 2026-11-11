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
