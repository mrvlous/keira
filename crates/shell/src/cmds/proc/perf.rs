// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

#![allow(unused_variables, unused_unsafe)]

//! Query Hardware Performance Monitoring Counters & CPU Profiling (Syscall 47 & 48).

use keira_io::vga;

pub fn run(parts: &mut core::str::SplitWhitespace) {
    if let Some("-h") | Some("--help") = parts.next() {
        unsafe {
            vga::print_str(
                "Usage: perf [stat|top|list]

",
            );
            vga::print_str(
                "Description:
  Query Hardware Performance Monitoring Counters & CPU Profiling (Syscall 47 & 48).

",
            );
            vga::print_str(
                "Options:
  -h, --help    Show this help message and exit
",
            );
        }
        return;
    }

    unsafe {
        vga::set_color(vga::Color::White, vga::Color::Black);
        vga::print_str("Hardware Performance Monitoring Counters (Syscall 47 & 48) ");
        vga::set_color(vga::Color::Yellow, vga::Color::Black);
        vga::print_str(
            "[PREVIEW]
",
        );
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        let _ = keira_arch::perf::sys_perf_event_open(1, 0, 1);
    }
}
