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
use keira_ipc::mqueue::get_mqueue_table;
use keira_ipc::shm::{get_sem_table, get_shm_table};

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

    let show_shm = args.has_flag('m', "shm");
    let show_sem = args.has_flag('s', "sem");
    let show_queues = args.has_flag('q', "queues");
    let show_all = args.has_flag('a', "all") || (!show_shm && !show_sem && !show_queues);

    unsafe {
        // 1. Shared Memory
        if show_all || show_shm {
            vga::set_color(vga::Color::White, vga::Color::Black);
            vga::print_str("------ Shared Memory Segments ------\n");
            vga::print_str("ID   KEY          BYTES   PHYS_FRAME   ATTACHES  OWNER\n");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);

            let table = get_shm_table();
            let mut count = 0;
            for seg in table.iter() {
                if seg.in_use {
                    vga::print_u64(seg.id as u64);
                    vga::print_str("    ");
                    vga::print_hex(seg.key as u64);
                    vga::print_str("   ");
                    vga::print_u64(seg.size_bytes as u64);
                    vga::print_str("    ");
                    vga::print_hex(seg.phys_frame);
                    vga::print_str("   ");
                    vga::print_u64(seg.attach_count as u64);
                    vga::print_str("         PID ");
                    vga::print_u64(seg.owner_pid as u64);
                    vga::print_str("\n");
                    count += 1;
                }
            }
            if count == 0 {
                vga::print_str("  (no active shared memory segments)\n");
            }
            vga::print_str("\n");
        }

        // 2. Semaphores
        if show_all || show_sem {
            vga::set_color(vga::Color::White, vga::Color::Black);
            vga::print_str("------ Semaphore Arrays ------\n");
            vga::print_str("ID   KEY          VALUE   WAITERS\n");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);

            let table = get_sem_table();
            let mut count = 0;
            for sem in table.iter() {
                if sem.in_use {
                    vga::print_u64(sem.id as u64);
                    vga::print_str("    ");
                    vga::print_hex(sem.key as u64);
                    vga::print_str("   ");
                    vga::print_u64(sem.value as u64);
                    vga::print_str("       ");
                    vga::print_u64(sem.waiters as u64);
                    vga::print_str("\n");
                    count += 1;
                }
            }
            if count == 0 {
                vga::print_str("  (no active semaphore arrays)\n");
            }
            vga::print_str("\n");
        }

        // 3. POSIX Message Queues
        if show_all || show_queues {
            vga::set_color(vga::Color::White, vga::Color::Black);
            vga::print_str("------ POSIX Message Queues ------\n");
            vga::print_str("ID   NAME             MSGS   MAX_MSGS  MSG_SIZE\n");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);

            let table = get_mqueue_table();
            let mut count = 0;
            for q in table.iter() {
                if q.in_use {
                    vga::print_u64(q.id as u64);
                    vga::print_str("    ");
                    vga::print_str(q.name_as_str());
                    // Pad name column to 17 chars
                    let len = q.name_len;
                    if len < 17 {
                        let pad = 17 - len;
                        for _ in 0..pad {
                            vga::print_str(" ");
                        }
                    } else {
                        vga::print_str(" ");
                    }
                    vga::print_u64(q.cur_msgs as u64);
                    vga::print_str("      ");
                    vga::print_u64(q.max_msg as u64);
                    vga::print_str("         ");
                    vga::print_u64(q.msg_size as u64);
                    vga::print_str(" B\n");
                    count += 1;
                }
            }
            if count == 0 {
                vga::print_str("  (no active message queues)\n");
            }
        }
    }
}
