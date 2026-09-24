// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Unit tests for DHCP client configuration.

use super::*;

#[test]
fn test_default_dhcp_config() {
    unsafe {
        assert!(SYSTEM_DHCP.configured);
        assert_eq!(SYSTEM_DHCP.ip_address, [10, 0, 2, 15]);
        assert_eq!(SYSTEM_DHCP.gateway, [10, 0, 2, 2]);
        assert_eq!(SYSTEM_DHCP.dns_server, [10, 0, 2, 3]);
    }
}
