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

use crate::args::CliArgs;
use keira_io::vga;
use keira_ipc::shm;

pub fn run(parts: &mut core::str::SplitWhitespace) {
    let args = CliArgs::parse(parts);

    if args.has_flag('h', "help") {
        unsafe {
            vga::set_color(vga::Color::White, vga::Color::Black);
            vga::print_str("Usage: ipcs [-m] [-s] [-a]\n\n");
            vga::print_str("Description:\n  Inspect active POSIX shared memory segments and counting semaphores.\n\n");
            vga::print_str("Options:\n");
            vga::print_str("  -m, --shm      Display shared memory segments\n");
            vga::print_str("  -s, --sem      Display counting semaphores\n");
            vga::print_str("  -a, --all      Display all active IPC facilities (default)\n");
            vga::print_str("  -h, --help     Show this help message and exit\n");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        }
        return;
    }

    unsafe {
        let _ = shm::sys_shm_sem(shm::SHM_CMD_INFO, 0, 0);
    }
}
