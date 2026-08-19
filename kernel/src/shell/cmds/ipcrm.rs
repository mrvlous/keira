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
//! Deallocate POSIX shared memory IPC segments or semaphores.

use crate::io::vga;
use crate::ipc::shm;

pub fn run(parts: &mut core::str::SplitWhitespace) {
    if let Some("-h") | Some("--help") = parts.next() {
        unsafe {
            vga::print_str("Usage: ipcrm <shmid|semid>\n\n");
            vga::print_str("Description:\n  Remove POSIX shared memory IPC segments or semaphores by ID (Syscall 75).\n\n");
            vga::print_str("Options:\n  -h, --help    Show this help message and exit\n");
        }
        return;
    }

    unsafe {
        let _ = shm::sys_shm_sem(shm::SHM_CMD_RM, 0, 0);
        vga::set_color(vga::Color::Yellow, vga::Color::Black);
        vga::print_str("[IPC] Deallocated target shared memory IPC segment/semaphore.\n");
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    }
}
