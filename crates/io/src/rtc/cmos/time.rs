// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Real-Time Clock date, time representation, and binary-coded decimal decoding.

/// Real-Time Clock timestamp representation.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RtcTime {
    /// Seconds component (0..59).
    pub second: u8,
    /// Minutes component (0..59).
    pub minute: u8,
    /// Hours component in 24-hour format (0..23).
    pub hour: u8,
    /// Day of the month (1..31).
    pub day: u8,
    /// Month of the year (1..12).
    pub month: u8,
    /// Full calendar year (e.g. 2026).
    pub year: u16,
}

/// Converts a binary-coded decimal (BCD) byte to standard binary integer.
#[inline]
pub fn bcd_to_bin(bcd: u8) -> u8 {
    ((bcd >> 4) * 10) + (bcd & 0x0F)
}

/// Converts a standard binary byte to binary-coded decimal (BCD).
#[inline]
pub fn bin_to_bcd(val: u8) -> u8 {
    ((val / 10) << 4) | (val % 10)
}
