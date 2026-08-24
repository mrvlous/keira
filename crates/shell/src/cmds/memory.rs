// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

#![allow(unused_variables, unused_unsafe)]

//! Implementation of the 'memory' shell command.

use crate::args::CliArgs;
use crate::executor::*;
use keira_io::vga;

pub fn run(parts: &mut core::str::SplitWhitespace) {
    let args = CliArgs::parse(parts);

    if args.has_flag('h', "help") {
        unsafe {
            vga::set_color(vga::Color::White, vga::Color::Black);
            vga::print_str("Usage: memory [-m] [-k] [-b] [-s]\n\n");
            vga::print_str(
                "Description:\n  Display physical frame allocator and kernel heap statistics.\n\n",
            );
            vga::print_str("Options:\n");
            vga::print_str("  -m, --mega     Format all memory values in Megabytes (MB)\n");
            vga::print_str("  -k, --kilo     Format all memory values in Kilobytes (KB)\n");
            vga::print_str("  -b, --bytes    Format all memory values in raw bytes\n");
            vga::print_str("  -s, --summary  Display compact one-line memory summary\n");
            vga::print_str("  -h, --help     Show this help message and exit\n");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        }
        return;
    }

    let (total, used, free) = unsafe {
        (
            heap_get_total() as u64,
            heap_get_used() as u64,
            heap_get_free() as u64,
        )
    };

    let (phys_total, phys_used, phys_free) = keira_mem::pmm::get_stats();

    if args.has_flag('s', "summary") {
        unsafe {
            vga::set_color(vga::Color::White, vga::Color::Black);
            vga::print_str("RAM: ");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            vga::print_u64(phys_used / (1024 * 1024));
            vga::print_str(" MB / ");
            vga::print_u64(phys_total / (1024 * 1024));
            vga::print_str(" MB | Heap: ");
            vga::print_u64(used / 1024);
            vga::print_str(" KB / ");
            vga::print_u64(total / 1024);
            vga::print_str(" KB free\n");
        }
        return;
    }

    let in_mega = args.has_flag('m', "mega");
    let in_bytes = args.has_flag('b', "bytes");

    unsafe {
        vga::set_color(vga::Color::White, vga::Color::Black);
        vga::print_str("Memory Statistics & Frame Allocator:\n");
        vga::print_str("REGION            TOTAL          USED           FREE\n");
        vga::print_str("----------------  -------------  -------------  -------------\n");

        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        // Kernel Heap Row
        vga::print_str("Kernel Heap       ");
        if in_bytes {
            print_padded_num(total, " B");
            print_padded_num(used, " B");
            print_padded_num(free, " B");
        } else if in_mega {
            print_padded_num(total / (1024 * 1024), " MB");
            print_padded_num(used / (1024 * 1024), " MB");
            print_padded_num(free / (1024 * 1024), " MB");
        } else {
            print_padded_num(total / 1024, " KB");
            print_padded_num(used / 1024, " KB");
            print_padded_num(free / 1024, " KB");
        }
        vga::print_str("\n");

        // Physical RAM Row
        vga::print_str("Physical RAM      ");
        if in_bytes {
            print_padded_num(phys_total, " B");
            print_padded_num(phys_used, " B");
            print_padded_num(phys_free, " B");
        } else if in_mega {
            print_padded_num(phys_total / (1024 * 1024), " MB");
            print_padded_num(phys_used / (1024 * 1024), " MB");
            print_padded_num(phys_free / (1024 * 1024), " MB");
        } else {
            print_padded_num(phys_total / 1024, " KB");
            print_padded_num(phys_used / 1024, " KB");
            print_padded_num(phys_free / 1024, " KB");
        }
        vga::print_str("\n\n");

        let (alloc_count, peak_bytes) = (heap_get_alloc_count() as u64, heap_get_peak() as u64);

        vga::set_color(vga::Color::White, vga::Color::Black);
        vga::print_str("Heap Allocator Statistics:\n");
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        vga::print_str("  Total Allocations : ");
        vga::print_u64(alloc_count);
        vga::print_str(" requests\n");
        vga::print_str("  Peak Heap Usage   : ");
        vga::print_u64(peak_bytes);
        vga::print_str(" bytes (");
        vga::print_u64(peak_bytes / 1024);
        vga::print_str(" KB)\n");
    }
}

unsafe fn print_padded_num(val: u64, unit: &str) {
    vga::print_u64(val);
    vga::print_str(unit);
    let mut len = unit.len();
    let mut temp = val;
    if temp == 0 {
        len += 1;
    } else {
        while temp > 0 {
            len += 1;
            temp /= 10;
        }
    }
    for _ in 0..(15usize.saturating_sub(len)) {
        vga::print_str(" ");
    }
}
