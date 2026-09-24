// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Unit tests for USB mass storage BOT commands and data structures.

use super::bot::{
    build_scsi_inquiry_cbw, build_scsi_read_capacity_cbw, CommandBlockWrapper, CommandStatusWrapper,
};

#[test]
fn test_usb_bot_struct_sizes() {
    assert_eq!(core::mem::size_of::<CommandBlockWrapper>(), 31);
    assert_eq!(core::mem::size_of::<CommandStatusWrapper>(), 13);
}

#[test]
fn test_scsi_cbw_construction() {
    let inq = build_scsi_inquiry_cbw(42);
    let inq_sig = inq.signature;
    let inq_tag = inq.tag;
    let inq_len = inq.data_transfer_length;
    assert_eq!(inq_sig, 0x43425355); // 'USBC'
    assert_eq!(inq_tag, 42);
    assert_eq!(inq_len, 36);
    assert_eq!(inq.cb[0], 0x12);

    let cap = build_scsi_read_capacity_cbw(99);
    let cap_sig = cap.signature;
    let cap_tag = cap.tag;
    let cap_len = cap.data_transfer_length;
    assert_eq!(cap_sig, 0x43425355);
    assert_eq!(cap_tag, 99);
    assert_eq!(cap_len, 8);
    assert_eq!(cap.cb[0], 0x25);
}
