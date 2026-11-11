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

//! Integration tests for Keira I/O subsystems and driver managers.

use super::*;

#[test]
fn test_io_subsystem_reexports_and_query() {
    // 1. Bus: PCI and PCIe
    let pci_count = unsafe { pci::PCI_DEVICE_COUNT };
    assert_eq!(pci_count, 0);

    pcie_init();
    unsafe {
        assert!(PCIE_INITIALIZED);
        let ecam = *core::ptr::addr_of!(PCIE_ECAM_BASE);
        assert_eq!(ecam, 0xE000_0000);
    }

    // 2. Storage: Block traits and NVMe
    nvme_ensure_initialized();
    let (ready, ns_count, cap_mb) = get_nvme_stats();
    assert!(ready);
    assert_eq!(ns_count, 1);
    assert_eq!(cap_mb, 1024);

    let ctrl = get_nvme_controller().expect("NVMe controller present");
    assert_eq!(ctrl.version, 0x0001_0400);

    // 3. VGA & Console
    vga_set_color(VgaColor::White, VgaColor::Black);
    assert_eq!(vga_get_cursor_row(), 0);
    assert_eq!(vga_get_cursor_col(), 0);

    // 4. USB Host & Storage
    usb_init();
    unsafe {
        assert!(USB_INITIALIZED);
    }
    let cbw = build_scsi_inquiry_cbw(1);
    let cbw_sig = cbw.signature;
    assert_eq!(cbw_sig, 0x43425355);

    // 5. TTY virtual terminals
    assert_eq!(get_active_tty(), 0);
    switch_tty(2);
    assert_eq!(get_active_tty(), 2);
    switch_tty(0);

    // 6. Logging sink multiplexer
    init_logging_sink();
    keira_core::klog_write(
        keira_core::KERN_INFO,
        "io/test",
        "io driver multiplexer integration verified",
    );
    let mut log_buf = [0u8; 256];
    let read_len = keira_core::klog_read_all(&mut log_buf);
    let log_str = core::str::from_utf8(&log_buf[..read_len]).expect("UTF-8");
    assert!(log_str.contains("io/test: io driver multiplexer integration verified\n"));
}
