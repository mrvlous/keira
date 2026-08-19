// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//!
//! Provides RBP/RSP pointer frame walking for kernel panic debugging backtraces
//! and process tracing via sys_ptrace.

use crate::io::{serial, vga};

/// Walk active kernel stack frame pointers using inline RBP register
#[inline(never)]
pub fn unwind_stack() {
    unsafe {
        let mut rbp: u64;
        core::arch::asm!("mov {}, rbp", out(reg) rbp);
        let mut rip: u64;
        core::arch::asm!("lea {}, [rip]", out(reg) rip);
        unwind_from_frame(rbp, rip);
    }
}

/// Walk kernel stack frame pointers from specific RBP/RIP context and print callstack backtrace
#[inline(never)]
pub unsafe fn unwind_from_frame(starting_rbp: u64, starting_rip: u64) {
    vga::set_color(vga::Color::LightRed, vga::Color::Black);
    vga::print_str("[UNWIND] KERNEL CALLSTACK BACKTRACE:\n");
    serial::print_str("[UNWIND] KERNEL CALLSTACK BACKTRACE:\n");
    vga::set_color(vga::Color::White, vga::Color::Black);

    vga::print_str("  [#0] RIP: 0x");
    print_hex(starting_rip);
    vga::print_str("\n");
    serial::print_str("  [#0] RIP: 0x");
    print_hex_serial(starting_rip);
    serial::print_str("\n");

    let mut rbp = starting_rbp;
    let mut depth = 1;

    while rbp != 0 && depth < 16 {
        let next_rbp_ptr = rbp as *const u64;
        let rip_ptr = (rbp + 8) as *const u64;

        if validate_ptr(rip_ptr as u64) && validate_ptr(next_rbp_ptr as u64) {
            let rip = *rip_ptr;
            let next_rbp = *next_rbp_ptr;

            vga::print_str("  [#");
            vga::print_u64(depth as u64);
            vga::print_str("] RIP: 0x");
            print_hex(rip);
            vga::print_str("\n");

            serial::print_str("  [#");
            print_decimal_serial(depth as u64);
            serial::print_str("] RIP: 0x");
            print_hex_serial(rip);
            serial::print_str("\n");

            if next_rbp <= rbp || next_rbp > 0x7FFFFFFFFFFF {
                break;
            }
            rbp = next_rbp;
        } else {
            break;
        }
        depth += 1;
    }
    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
}

fn validate_ptr(ptr: u64) -> bool {
    ptr >= 0x100000 && ptr < 0x7FFFFFFFFFFF && (ptr % 8 == 0)
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

fn print_hex_serial(val: u64) {
    let hex_chars = b"0123456789ABCDEF";
    let mut buf = [0u8; 16];
    for i in 0..16 {
        buf[15 - i] = hex_chars[((val >> (i * 4)) & 0xF) as usize];
    }
    if let Ok(s) = core::str::from_utf8(&buf) {
        serial::print_str(s);
    }
}

fn print_decimal_serial(mut val: u64) {
    if val == 0 {
        serial::print_str("0");
        return;
    }
    let mut buf = [0u8; 20];
    let mut i = 0;
    while val > 0 {
        buf[i] = b'0' + (val % 10) as u8;
        val /= 10;
        i += 1;
    }
    for idx in 0..i {
        let char_buf = [buf[i - 1 - idx]];
        if let Ok(s) = core::str::from_utf8(&char_buf) {
            serial::print_str(s);
        }
    }
}
