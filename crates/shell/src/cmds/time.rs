// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

#![allow(unused_variables, unused_unsafe)]

//!
//! Implementation of the 'time' shell command.

use crate::executor::*;
use keira_io::vga;

pub fn run(parts: &mut core::str::SplitWhitespace) {
    unsafe {
        if let Some("-h") | Some("--help") = parts.next() {
            vga::print_str("Usage: time\n\n");
            vga::print_str("Description:\n  Query CMOS Real-Time Clock (RTC) hardware registers to display system date and UTC time.\n\n");
            vga::print_str("Options:\n  -h, --help    Show this help message and exit\n");
            return;
        }

        let mut time = RtcTime {
            second: 0,
            minute: 0,
            hour: 0,
            day: 0,
            month: 0,
            year: 0,
        };
        unsafe {
            rtc_get_time(&mut time as *mut RtcTime);
        }
        vga::set_color(vga::Color::White, vga::Color::Black);
        vga::print_str("Date: ");
        vga::set_color(vga::Color::White, vga::Color::Black);
        vga::print_u64(time.year as u64);
        vga::print_str("-");
        print_2digit(time.month as u64);
        vga::print_str("-");
        print_2digit(time.day as u64);
        vga::print_str(" ");
        print_2digit(time.hour as u64);
        vga::print_str(":");
        print_2digit(time.minute as u64);
        vga::print_str(":");
        print_2digit(time.second as u64);
        vga::print_str(" UTC\n");
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    }
}
