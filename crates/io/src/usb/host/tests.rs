// SPDX-License-Identifier: GPL-2.0-only

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
