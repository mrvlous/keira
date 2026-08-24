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
//! Implementation of the 'drives' shell command.

use crate::args::CliArgs;
use keira_io::vga;

pub fn run(parts: &mut core::str::SplitWhitespace) {
    let args = CliArgs::parse(parts);

    if args.has_flag('h', "help") {
        unsafe {
            vga::set_color(vga::Color::White, vga::Color::Black);
            vga::print_str("Usage: drives [-d] [-s]\n\n");
            vga::print_str("Description:\n  List all registered storage drives, capacity in KB, and mount states.\n\n");
            vga::print_str("Options:\n");
            vga::print_str("  -d, --detail   Display sector counts and block controller info\n");
            vga::print_str("  -s, --summary  Display total storage capacity summary\n");
            vga::print_str("  -h, --help     Show this help message and exit\n");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        }
        return;
    }

    unsafe {
        let mut total_kb = 0u64;
        let mut dev_count = 0u64;

        if !args.has_flag('s', "summary") {
            vga::set_color(vga::Color::White, vga::Color::Black);
            if args.has_flag('d', "detail") {
                vga::print_str("NAME      TYPE      SIZE (KB)   SECTORS     BLOCKSZ  STATUS\n");
                vga::print_str("--------  --------  ----------  ----------  -------  ---------\n");
            } else {
                vga::print_str("NAME      TYPE      SIZE (KB)   STATUS\n");
                vga::print_str("--------  --------  ----------  ---------\n");
            }
        }

        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        keira_io::block::for_each_device(|dev, is_mounted| {
            let name = dev.get_name();
            let sectors = dev.get_size_sectors() as u64;
            let size_kb = sectors / 2;
            total_kb += size_kb;
            dev_count += 1;

            if args.has_flag('s', "summary") {
                return;
            }

            vga::set_color(vga::Color::White, vga::Color::Black);
            vga::print_str(name);
            for _ in 0..(10usize.saturating_sub(name.len())) {
                vga::print_str(" ");
            }

            let type_str = if name.starts_with("ram") {
                "RAM Disk"
            } else if name.starts_with("ahci") {
                "SATA Disk"
            } else {
                "IDE Disk"
            };
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            vga::print_str(type_str);
            for _ in 0..(10usize.saturating_sub(type_str.len())) {
                vga::print_str(" ");
            }

            vga::print_u64(size_kb);
            let mut s_len = 0;
            let mut temp = size_kb;
            if temp == 0 {
                s_len = 1;
            } else {
                while temp > 0 {
                    s_len += 1;
                    temp /= 10;
                }
            }
            for _ in 0..(12usize.saturating_sub(s_len)) {
                vga::print_str(" ");
            }

            if args.has_flag('d', "detail") {
                vga::print_u64(sectors);
                let mut sec_len = 0;
                let mut temp_sec = sectors;
                if temp_sec == 0 {
                    sec_len = 1;
                } else {
                    while temp_sec > 0 {
                        sec_len += 1;
                        temp_sec /= 10;
                    }
                }
                for _ in 0..(12usize.saturating_sub(sec_len)) {
                    vga::print_str(" ");
                }
                vga::print_str("512 B    ");
            }

            if is_mounted {
                vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                vga::print_str("[Mounted]\n");
            } else {
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                vga::print_str("[Unmounted]\n");
            }
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        });

        if args.has_flag('s', "summary") {
            vga::set_color(vga::Color::White, vga::Color::Black);
            vga::print_str("Storage Summary: ");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            vga::print_u64(dev_count);
            vga::print_str(" drives (");
            vga::print_u64(total_kb / 1024);
            vga::print_str(" MB total capacity)\n");
        }
    }
}
