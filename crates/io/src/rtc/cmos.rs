// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! CMOS Real-Time Clock (RTC) driver in pure Rust.

use keira_arch::cpu::{inb, io_wait, outb};

pub const CMOS_ADDRESS_PORT: u16 = 0x70;
pub const CMOS_DATA_PORT: u16 = 0x71;

const RTC_REG_SECONDS: u8 = 0x00;
const RTC_REG_MINUTES: u8 = 0x02;
const RTC_REG_HOURS: u8 = 0x04;
const RTC_REG_DAY: u8 = 0x07;
const RTC_REG_MONTH: u8 = 0x08;
const RTC_REG_YEAR: u8 = 0x09;
const RTC_REG_STATUS_A: u8 = 0x0A;
const RTC_REG_STATUS_B: u8 = 0x0B;

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct RtcTime {
    pub second: u8,
    pub minute: u8,
    pub hour: u8,
    pub day: u8,
    pub month: u8,
    pub year: u16,
}

fn cmos_read(reg: u8) -> u8 {
    unsafe {
        outb(CMOS_ADDRESS_PORT, reg);
        io_wait();
        inb(CMOS_DATA_PORT)
    }
}

fn rtc_update_in_progress() -> bool {
    (cmos_read(RTC_REG_STATUS_A) & 0x80) != 0
}

fn bcd_to_bin(bcd: u8) -> u8 {
    ((bcd >> 4) * 10) + (bcd & 0x0F)
}

/// Initialize CMOS RTC hardware subsystem.
pub fn init() {}

/// Read current date and time registers from CMOS RTC.
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

// C-compatible export
#[no_mangle]
pub extern "C" fn rtc_get_time(time_ptr: *mut RtcTime) {
    if !time_ptr.is_null() {
        let t = get_time();
        unsafe {
            *time_ptr = t;
        }
    }
}

#[no_mangle]
pub extern "C" fn rtc_init() {
    init();
}
