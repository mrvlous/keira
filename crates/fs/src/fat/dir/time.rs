// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Real-time clock timestamp translation into FAT time and date formats.

/// Retrieves current real-time clock timestamp encoded in standard 16-bit FAT format.
pub fn get_rtc_fat_time_date() -> (u16, u16) {
    let time = keira_io::rtc::cmos::get_time();
    let fat_time =
        ((time.hour as u16) << 11) | ((time.minute as u16) << 5) | ((time.second as u16) / 2);
    let year_offset = time.year.saturating_sub(1980);
    let fat_date = (year_offset << 9) | ((time.month as u16) << 5) | (time.day as u16);
    (fat_time, fat_date)
}
