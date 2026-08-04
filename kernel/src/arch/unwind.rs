// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Keira Kernel: Kernel Callstack Unwinder & Process Tracing Engine
//!
//! Provides RBP/RSP pointer frame walking for kernel panic debugging backtraces
//! and process tracing via sys_ptrace.

use crate::io::vga;

/// Walk kernel stack frame pointers and print callstack backtrace
pub fn unwind_stack() {
    unsafe {
        let mut rbp: u64;
        core::arch::asm!("mov {}, rbp", out(reg) rbp);

        vga::set_color(vga::Color::LightRed, vga::Color::Black);
        vga::print_str("[KERNEL CALLSTACK BACKTRACE]\n");
        vga::set_color(vga::Color::White, vga::Color::Black);

        let mut depth = 0;
        while rbp != 0 && depth < 8 {
            let rip_ptr = (rbp + 8) as *const u64;
            if validate_ptr(rip_ptr as u64) {
                let rip = *rip_ptr;
                vga::print_str("  [#");
                vga::print_u64(depth as u64);
                vga::print_str("] RIP: 0x");
                print_hex(rip);
                vga::print_str("\n");
            } else {
                break;
            }
            let next_rbp_ptr = rbp as *const u64;
            if validate_ptr(next_rbp_ptr as u64) {
                rbp = *next_rbp_ptr;
            } else {
                break;
            }
            depth += 1;
        }
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    }
}

fn validate_ptr(ptr: u64) -> bool {
    ptr >= 0x100000 && ptr < 0x7FFFFFFFFFFF
}

fn print_hex(val: u64) {
    let hex_chars = b"0123456789ABCDEF";
    let mut buf = [0u8; 16];
    for i in 0..16 {
        buf[15 - i] = hex_chars[((val >> (i * 4)) & 0xF) as usize];
    }
    if let Ok(s) = core::str::from_utf8(&buf) {
        vga::print_str(s);
    }
}
