// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

#![allow(unused_variables, unused_unsafe)]

//! Query System V / POSIX IPC facilities (Shared Memory, Semaphores, Message Queues).

use crate::args::CliArgs;
use keira_io::vga;

pub fn run(parts: &mut core::str::SplitWhitespace) {
    let args = CliArgs::parse(parts);
    if args.has_flag('h', "help") {
        unsafe {
            vga::set_color(vga::Color::White, vga::Color::Black);
            vga::print_str("Usage: ipcs [-m] [-s] [-q] [-a]\n\n");
            vga::print_str(
                "Description:\n  Query status of System V and POSIX IPC facilities (Syscall 38-40).\n\n",
            );
            vga::print_str("Options:\n");
            vga::print_str("  -m, --shm      Display active Shared Memory segments\n");
            vga::print_str("  -s, --sem      Display active Semaphore arrays\n");
            vga::print_str("  -q, --queues   Display active Message Queues\n");
            vga::print_str("  -a, --all      Display all IPC facilities (default)\n");
            vga::print_str("  -h, --help     Show this help message and exit\n");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        }
        return;
    }

    unsafe {
        vga::set_color(vga::Color::White, vga::Color::Black);
        vga::print_str("System V / POSIX IPC Facilities (Syscall 38-40) ");
        vga::set_color(vga::Color::Yellow, vga::Color::Black);
        vga::print_str("[PREVIEW]\n");
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        let _ = keira_ipc::shm::sys_shm_sem(0, 0, 0);
        vga::print_str("  Shared Memory : 0 active segments\n");
        vga::print_str("  Semaphores    : 0 active sets\n");
    }
}
