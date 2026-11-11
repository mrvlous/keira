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

//! Unit tests for USB host controller structure properties.

use super::discovery::UsbControllerInfo;

#[test]
fn test_usb_controller_info() {
    let info = UsbControllerInfo {
        bus: 0,
        slot: 1,
        func: 2,
        interface_type: 0x30,
        bar0: 0xFEE00000,
        vendor_id: 0x8086,
        device_id: 0x1234,
    };
    assert_eq!(info.interface_type, 0x30);
    assert_eq!(info.vendor_id, 0x8086);
}
