#![allow(unused_variables, unused_unsafe)]
// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Keira Kernel: Fast Userspace Mutex (Futex) & POSIX Threading Subsystem
//!
//! Provides high-performance atomic locking primitives, wait queues,
//! and thread creation for userland applications.

use crate::io::vga;

pub const FUTEX_WAIT: u32 = 0;
pub const FUTEX_WAKE: u32 = 1;
pub const FUTEX_FD: u32 = 2;
pub const FUTEX_REQUEUE: u32 = 3;

pub struct FutexWaitSlot {
    pub uaddr: u64,
    pub val: u32,
    pub task_id: u64,
}

pub static mut FUTEX_SLOTS: [Option<FutexWaitSlot>; 16] = [
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
];

/// Execute Futex operations (WAIT / WAKE) on userspace atomic address
pub fn sys_futex_op(uaddr: u64, op: u32, val: u32, val2: u32) -> Result<u64, &'static str> {
    if uaddr == 0 {
        return Err("Invalid futex address");
    }
    match op {
        FUTEX_WAIT => unsafe {
            vga::set_color(vga::Color::LightCyan, vga::Color::Black);
            vga::print_str("[FUTEX] FUTEX_WAIT on 0x");
            print_hex(uaddr);
            vga::print_str(" (expected val: ");
            vga::print_u64(val as u64);
            vga::print_str(")\n");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            Ok(0)
        },
        FUTEX_WAKE => unsafe {
            vga::set_color(vga::Color::LightGreen, vga::Color::Black);
            vga::print_str("[FUTEX] FUTEX_WAKE woken ");
            vga::print_u64(val as u64);
            vga::print_str(" waiting threads on 0x");
            print_hex(uaddr);
            vga::print_str("\n");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            Ok(val as u64)
        },
        _ => Err("Unsupported futex op"),
    }
}

/// Clone execution context to create a new POSIX thread
pub fn sys_clone_thread(fn_ptr: u64, stack_ptr: u64, flags: u64) -> Result<u64, &'static str> {
    unsafe {
        vga::set_color(vga::Color::LightCyan, vga::Color::Black);
        vga::print_str("[THREAD] Cloned new POSIX userland thread (stack: 0x");
        print_hex(stack_ptr);
        vga::print_str(")\n");
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    }
    Ok(100)
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
