// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Unit tests for CMOS Real-Time Clock data structures and BCD conversions.

use super::time::{bcd_to_bin, bin_to_bcd, RtcTime};

#[test]
fn test_bcd_conversions() {
    assert_eq!(bcd_to_bin(0x00), 0);
    assert_eq!(bcd_to_bin(0x09), 9);
    assert_eq!(bcd_to_bin(0x10), 10);
    assert_eq!(bcd_to_bin(0x26), 26);
    assert_eq!(bcd_to_bin(0x59), 59);

    assert_eq!(bin_to_bcd(0), 0x00);
    assert_eq!(bin_to_bcd(9), 0x09);
    assert_eq!(bin_to_bcd(10), 0x10);
    assert_eq!(bin_to_bcd(26), 0x26);
    assert_eq!(bin_to_bcd(59), 0x59);
}

#[test]
fn test_rtc_time_default() {
    let t = RtcTime::default();
    assert_eq!(t.second, 0);
    assert_eq!(t.minute, 0);
    assert_eq!(t.hour, 0);
    assert_eq!(t.day, 0);
    assert_eq!(t.month, 0);
    assert_eq!(t.year, 0);
}

#[test]
fn test_rtc_time_layout() {
    let t = RtcTime {
        second: 45,
        minute: 30,
        hour: 14,
        day: 24,
        month: 9,
        year: 2026,
    };
    assert_eq!(t.year, 2026);
    assert_eq!(t.month, 9);
    assert_eq!(t.day, 24);
    assert_eq!(t.hour, 14);
    assert_eq!(t.minute, 30);
    assert_eq!(t.second, 45);
}
