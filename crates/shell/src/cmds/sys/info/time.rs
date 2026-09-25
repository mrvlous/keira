// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Implementation of the 'time' shell command.

use crate::args::CliArgs;
use crate::executor::*;
use keira_io::vga;

pub fn run(parts: &mut core::str::SplitWhitespace) {
    let args = CliArgs::parse(parts);

    if args.has_flag('h', "help") {
        {
            vga::print_str("Usage: time [-d] [-t]\n\n");
            vga::print_str(
                "Description:\n  Display real-time clock (RTC) current system date and time.\n",
            );
            vga::print_str(
                "Options:\n  -d, --date    Display date only\n  -t, --time    Display time only\n",
            );
        }
        return;
    }

    {
        let time = keira_io::rtc::get_time();

        vga::set_color(vga::Color::White, vga::Color::Black);
        if args.has_flag('d', "date") {
            vga::print_str("Date: ");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            vga::print_u64(time.year as u64);
            vga::print_str("-");
            print_2digit(time.month as u64);
            vga::print_str("-");
            print_2digit(time.day as u64);
            vga::print_str("\n");
        } else if args.has_flag('t', "time") {
            vga::print_str("Time: ");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            print_2digit(time.hour as u64);
            vga::print_str(":");
            print_2digit(time.minute as u64);
            vga::print_str(":");
            print_2digit(time.second as u64);
            vga::print_str(" UTC\n");
        } else {
            vga::print_str("Date: ");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
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
        }
    }
}
