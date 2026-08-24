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
//! Query Fast Userspace Mutex wait queue status (Syscall 40).

use keira_io::vga;

pub fn run(parts: &mut core::str::SplitWhitespace) {
    if let Some("-h") | Some("--help") = parts.next() {
        unsafe {
            vga::print_str("Usage: futex [status]\n\n");
            vga::print_str("Description:\n  Query Fast Userspace Mutex (Futex) wait queue and locking status (Syscall 40).\n\n");
            vga::print_str("Options:\n  -h, --help    Show this help message and exit\n");
        }
        return;
    }

    unsafe {
        vga::set_color(vga::Color::White, vga::Color::Black);
        vga::print_str("Fast Userspace Mutex (Futex) Subsystem (Syscall 40):\n");
        let _ = keira_ipc::futex::sys_futex(
            0x400000 as *mut u32,
            keira_ipc::futex::FUTEX_WAKE,
            1,
            0,
            core::ptr::null_mut(),
            0,
        );
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    }
}
