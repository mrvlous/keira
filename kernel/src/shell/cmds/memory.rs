#![allow(unused_variables, unused_unsafe)]
// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Keira Kernel: Shell Command 'memory'
//!
//! Implementation of the 'memory' shell command.

use crate::io::vga;
use crate::shell::executor::*;

pub fn run(parts: &mut core::str::SplitWhitespace) {
    unsafe {
        if let Some("-h") | Some("--help") = parts.next() {
            vga::print_str("Usage: memory\n\n");
            vga::print_str("Description:\n  Display kernel C bump heap allocator statistics, peak consumption, allocation request counts, and physical RAM frame stats.\n\n");
            vga::print_str("Options:\n  -h, --help    Show this help message and exit\n");
            return;
        }

        let total = unsafe { heap_get_total() } as u64;
        let used = unsafe { heap_get_used() } as u64;
        let free = unsafe { heap_get_free() } as u64;

        let (phys_total, phys_used, phys_free) = crate::mem::pmm::get_stats();

        vga::set_color(vga::Color::LightBlue, vga::Color::Black);
        vga::print_str("Memory Statistics:\n");
        vga::print_str("REGION            TOTAL          USED           FREE\n");

        vga::set_color(vga::Color::White, vga::Color::Black);
        // Kernel Heap Row
        vga::print_str("Kernel Heap       ");
        vga::print_u64(total / 1024);
        vga::print_str(" KB");
        let mut total_len = 0;
        let mut temp = total / 1024;
        while temp > 0 {
            total_len += 1;
            temp /= 10;
        }
        for _ in 0..(12 - total_len) {
            vga::print_str(" ");
        }

        vga::print_u64(used / 1024);
        vga::print_str(" KB");
        let mut used_len = 0;
        let mut temp = used / 1024;
        if temp == 0 {
            used_len = 1;
        } else {
            while temp > 0 {
                used_len += 1;
                temp /= 10;
            }
        }
        for _ in 0..(12 - used_len) {
            vga::print_str(" ");
        }

        vga::print_u64(free / 1024);
        vga::print_str(" KB\n");

        // Physical RAM Row
        vga::print_str("Physical RAM      ");
        vga::print_u64(phys_total / (1024 * 1024));
        vga::print_str(" MB");
        let mut phys_total_len = 0;
        let mut temp = phys_total / (1024 * 1024);
        while temp > 0 {
            phys_total_len += 1;
            temp /= 10;
        }
        for _ in 0..(12 - phys_total_len) {
            vga::print_str(" ");
        }

        vga::print_u64(phys_used / 1024);
        vga::print_str(" KB");
        let mut phys_used_len = 0;
        let mut temp = phys_used / 1024;
        if temp == 0 {
            phys_used_len = 1;
        } else {
            while temp > 0 {
                phys_used_len += 1;
                temp /= 10;
            }
        }
        for _ in 0..(12 - phys_used_len) {
            vga::print_str(" ");
        }

        vga::print_u64(phys_free / 1024);
        vga::print_str(" KB\n\n");

        let alloc_count = unsafe { heap_get_alloc_count() } as u64;
        let peak_bytes = unsafe { heap_get_peak() } as u64;

        vga::set_color(vga::Color::LightCyan, vga::Color::Black);
        vga::print_str("Heap Allocator Statistics:\n");
        vga::set_color(vga::Color::White, vga::Color::Black);
        vga::print_str("  Total Allocations : ");
        vga::print_u64(alloc_count);
        vga::print_str(" requests\n");
        vga::print_str("  Peak Heap Usage   : ");
        vga::print_u64(peak_bytes);
        vga::print_str(" bytes (");
        vga::print_u64(peak_bytes / 1024);
        vga::print_str(" KB)\n");

        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    }
}
