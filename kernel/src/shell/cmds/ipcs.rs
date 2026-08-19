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
//! Query active POSIX shared memory segments and semaphores.

use crate::io::vga;
use crate::ipc::shm;

pub fn run(parts: &mut core::str::SplitWhitespace) {
    if let Some("-h") | Some("--help") = parts.next() {
        unsafe {
            vga::print_str("Usage: ipcs [-m|-s|-a]\n\n");
            vga::print_str("Description:\n  Inspect active POSIX shared memory IPC segments and semaphores (Syscall 75).\n\n");
            vga::print_str("Options:\n  -h, --help    Show this help message and exit\n");
        }
        return;
    }

    unsafe {
        let _ = shm::sys_shm_sem(shm::SHM_CMD_INFO, 0, 0);
    }
}
