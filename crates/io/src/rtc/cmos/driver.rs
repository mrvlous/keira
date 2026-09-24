// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! CMOS Real-Time Clock (RTC) port I/O hardware driver.

use super::time::{bcd_to_bin, RtcTime};
use keira_arch::cpu::{inb, io_wait, outb};

/// Primary I/O port address for CMOS register selection.
pub const CMOS_ADDRESS_PORT: u16 = 0x70;

/// Primary I/O port address for CMOS register data transfer.
pub const CMOS_DATA_PORT: u16 = 0x71;

/// CMOS internal RTC register index for seconds.
pub const RTC_REG_SECONDS: u8 = 0x00;

/// CMOS internal RTC register index for minutes.
pub const RTC_REG_MINUTES: u8 = 0x02;

/// CMOS internal RTC register index for hours.
pub const RTC_REG_HOURS: u8 = 0x04;

/// CMOS internal RTC register index for day of month.
pub const RTC_REG_DAY: u8 = 0x07;

/// CMOS internal RTC register index for month.
pub const RTC_REG_MONTH: u8 = 0x08;

/// CMOS internal RTC register index for year.
pub const RTC_REG_YEAR: u8 = 0x09;

/// CMOS internal RTC register index for Status Register A.
pub const RTC_REG_STATUS_A: u8 = 0x0A;

/// CMOS internal RTC register index for Status Register B.
pub const RTC_REG_STATUS_B: u8 = 0x0B;

/// Reads an 8-bit value from the specified CMOS register index.
#[inline]
pub fn cmos_read(reg: u8) -> u8 {
    // Port I/O requires unsafe block, caller guarantees valid port access.
    unsafe {
        outb(CMOS_ADDRESS_PORT, reg);
        io_wait();
        inb(CMOS_DATA_PORT)
    }
}

/// Checks whether the RTC hardware is currently updating its internal counters.
#[inline]
pub fn rtc_update_in_progress() -> bool {
    (cmos_read(RTC_REG_STATUS_A) & 0x80) != 0
}

/// Initializes the CMOS RTC hardware subsystem.
pub fn init() {}

/// Reads current date and time registers from the CMOS RTC.
///
/// Implements dual-read consistency checks to guard against reading mid-update
/// rolled-over values, followed by BCD and 12-hour/24-hour canonical translation.
pub fn get_time() -> RtcTime {
    while rtc_update_in_progress() {}

    let mut second = cmos_read(RTC_REG_SECONDS);
    let mut minute = cmos_read(RTC_REG_MINUTES);
    let mut hour = cmos_read(RTC_REG_HOURS);
    let mut day = cmos_read(RTC_REG_DAY);
    let mut month = cmos_read(RTC_REG_MONTH);
    let mut year = cmos_read(RTC_REG_YEAR);

    while rtc_update_in_progress() {}
    let second2 = cmos_read(RTC_REG_SECONDS);
    let minute2 = cmos_read(RTC_REG_MINUTES);
    let hour2 = cmos_read(RTC_REG_HOURS);
    let day2 = cmos_read(RTC_REG_DAY);
    let month2 = cmos_read(RTC_REG_MONTH);
    let year2 = cmos_read(RTC_REG_YEAR);

    if second != second2
        || minute != minute2
        || hour != hour2
        || day != day2
        || month != month2
        || year != year2
    {
        while rtc_update_in_progress() {}
        second = cmos_read(RTC_REG_SECONDS);
        minute = cmos_read(RTC_REG_MINUTES);
        hour = cmos_read(RTC_REG_HOURS);
        day = cmos_read(RTC_REG_DAY);
        month = cmos_read(RTC_REG_MONTH);
        year = cmos_read(RTC_REG_YEAR);
    }

    let status_b = cmos_read(RTC_REG_STATUS_B);

    if (status_b & 0x04) == 0 {
        second = bcd_to_bin(second);
        minute = bcd_to_bin(minute);
        hour = bcd_to_bin(hour & 0x7F) | (hour & 0x80);
        day = bcd_to_bin(day);
        month = bcd_to_bin(month);
        year = bcd_to_bin(year);
    }

    if (status_b & 0x02) == 0 && (hour & 0x80) != 0 {
        hour = ((hour & 0x7F) + 12) % 24;
    }

    RtcTime {
        second,
        minute,
        hour,
        day,
        month,
        year: 2000 + (year as u16),
    }
}

/// Foreign function export for retrieving RTC time into a C-compatible structure pointer.
///
/// # Safety
///
/// Caller must ensure `time_ptr` points to valid, writable, properly aligned `RtcTime` memory.
#[no_mangle]
pub unsafe extern "C" fn rtc_get_time(time_ptr: *mut RtcTime) {
    if !time_ptr.is_null() {
        let t = get_time();
        *time_ptr = t;
    }
}

/// Foreign function export for initializing the RTC subsystem.
#[no_mangle]
pub extern "C" fn rtc_init() {
    init();
}
