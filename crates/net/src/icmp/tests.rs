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

//! Unit tests for ICMP echo module.

use super::*;

#[test]
fn test_icmp_offline_ping() {
    unsafe {
        crate::driver::e1000::E1000_FOUND = false;
        assert!(send_ping("127.0.0.1").is_ok());
        let res = send_ping("10.0.2.2");
        assert!(res.is_err());
    }
}
