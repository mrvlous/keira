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

//! Motorola MC146818 CMOS Real-Time Clock (RTC) driver and date-time decoding.

pub mod driver {
    use super::time::{bcd_to_bin, RtcTime};
    use keira_arch::cpu::{inb, io_wait, outb};
    
    pub const CMOS_ADDRESS_PORT: u16 = 0x70;
    
    pub const CMOS_DATA_PORT: u16 = 0x71;
    
    pub const RTC_REG_SECONDS: u8 = 0x00;
    
    pub const RTC_REG_MINUTES: u8 = 0x02;
    
    pub const RTC_REG_HOURS: u8 = 0x04;
    
    pub const RTC_REG_DAY: u8 = 0x07;
    
    pub const RTC_REG_MONTH: u8 = 0x08;
    
    pub const RTC_REG_YEAR: u8 = 0x09;
    
    pub const RTC_REG_STATUS_A: u8 = 0x0A;
    
    pub const RTC_REG_STATUS_B: u8 = 0x0B;
    
    #[inline]
    pub fn cmos_read(reg: u8) -> u8 {
        // Port I/O requires unsafe block, caller guarantees valid port access.
        unsafe {
            outb(CMOS_ADDRESS_PORT, reg);
            io_wait();
            inb(CMOS_DATA_PORT)
        }
    }
    
    #[inline]
    pub fn rtc_update_in_progress() -> bool {
        (cmos_read(RTC_REG_STATUS_A) & 0x80) != 0
    }
    
    pub fn init() {}
    
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
    
    #[no_mangle]
    pub unsafe extern "C" fn rtc_get_time(time_ptr: *mut RtcTime) {
        if !time_ptr.is_null() {
            let t = get_time();
            *time_ptr = t;
        }
    }
    
    #[no_mangle]
    pub extern "C" fn rtc_init() {
        init();
    }
}

pub mod time {
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
    
    #[inline]
    pub fn bcd_to_bin(bcd: u8) -> u8 {
        ((bcd >> 4) * 10) + (bcd & 0x0F)
    }
    
    #[inline]
    pub fn bin_to_bcd(val: u8) -> u8 {
        ((val / 10) << 4) | (val % 10)
    }
}

pub use self::driver::*;
pub use self::time::*;

#[cfg(test)]
mod tests {
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
}
