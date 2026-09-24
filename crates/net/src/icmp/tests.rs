// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Unit tests for ICMP echo module.

use super::*;

#[test]
fn test_icmp_offline_ping() {
    unsafe {
        crate::driver::e1000::E1000_FOUND = false;
        let res = send_ping("10.0.2.2");
        assert!(res.is_err());
    }
}
