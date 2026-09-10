// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

#![allow(unused_variables, unused_unsafe)]

//! Remove System V / POSIX IPC facilities (Shared Memory, Semaphores, Message Queues).

use keira_io::vga;
use keira_ipc::mqueue::{mq_unlink, mq_unlink_by_id};
use keira_ipc::shm::{remove_sem, remove_shm};

pub fn run(parts: &mut core::str::SplitWhitespace) {
    let flag = match parts.next() {
        Some("-h") | Some("--help") => {
            unsafe {
                vga::set_color(vga::Color::White, vga::Color::Black);
                vga::print_str("Usage: ipcrm [-m <shmid>] [-s <semid>] [-q <msqid|name>]\n\n");
                vga::print_str(
                    "Description:\n  Remove System V and POSIX IPC facilities from kernel memory (Syscall 41 & 42).\n\n",
                );
                vga::print_str("Options:\n");
                vga::print_str("  -m, --shm <shmid>       Remove Shared Memory segment by ID\n");
                vga::print_str("  -s, --sem <semid>       Remove Semaphore array by ID\n");
                vga::print_str(
                    "  -q, --queue <id|name>   Remove POSIX Message Queue by ID or Name\n",
                );
                vga::print_str("  -h, --help              Show this help message and exit\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            }
            return;
        }
        Some(f) => f,
        None => {
            unsafe {
                vga::set_color(vga::Color::LightRed, vga::Color::Black);
                vga::print_str(
                    "Error: Option required. Usage: ipcrm [-m <id>] [-s <id>] [-q <id|name>]\n",
                );
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            }
            return;
        }
    };

    let target = match parts.next() {
        Some(t) => t,
        None => {
            unsafe {
                vga::set_color(vga::Color::LightRed, vga::Color::Black);
                vga::print_str("Error: Missing target ID or name for option '");
                vga::print_str(flag);
                vga::print_str("'.\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            }
            return;
        }
    };

    unsafe {
        match flag {
            "-m" | "--shm" => {
                if let Ok(id) = parse_u32(target) {
                    match remove_shm(id) {
                        Ok(_) => {
                            vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                            vga::print_str("[OK] ");
                            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                            vga::print_str("Removed Shared Memory segment #");
                            vga::print_u64(id as u64);
                            vga::print_str("\n");
                        }
                        Err(e) => {
                            vga::set_color(vga::Color::LightRed, vga::Color::Black);
                            vga::print_str("Error: Failed to remove Shared Memory segment #");
                            vga::print_u64(id as u64);
                            vga::print_str(" (");
                            vga::print_str(e);
                            vga::print_str(")\n");
                            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                        }
                    }
                } else {
                    vga::set_color(vga::Color::LightRed, vga::Color::Black);
                    vga::print_str("Error: Invalid numeric Shared Memory ID '");
                    vga::print_str(target);
                    vga::print_str("'\n");
                    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                }
            }
            "-s" | "--sem" => {
                if let Ok(id) = parse_u32(target) {
                    match remove_sem(id) {
                        Ok(_) => {
                            vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                            vga::print_str("[OK] ");
                            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                            vga::print_str("Removed Semaphore array #");
                            vga::print_u64(id as u64);
                            vga::print_str("\n");
                        }
                        Err(e) => {
                            vga::set_color(vga::Color::LightRed, vga::Color::Black);
                            vga::print_str("Error: Failed to remove Semaphore array #");
                            vga::print_u64(id as u64);
                            vga::print_str(" (");
                            vga::print_str(e);
                            vga::print_str(")\n");
                            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                        }
                    }
                } else {
                    vga::set_color(vga::Color::LightRed, vga::Color::Black);
                    vga::print_str("Error: Invalid numeric Semaphore ID '");
                    vga::print_str(target);
                    vga::print_str("'\n");
                    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                }
            }
            "-q" | "--queue" => {
                let res = if let Ok(id) = parse_u32(target) {
                    mq_unlink_by_id(id)
                } else {
                    mq_unlink(target)
                };

                match res {
                    Ok(_) => {
                        vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                        vga::print_str("[OK] ");
                        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                        vga::print_str("Unlinked POSIX Message Queue '");
                        vga::print_str(target);
                        vga::print_str("'\n");
                    }
                    Err(e) => {
                        vga::set_color(vga::Color::LightRed, vga::Color::Black);
                        vga::print_str("Error: Failed to unlink POSIX Message Queue '");
                        vga::print_str(target);
                        vga::print_str("' (");
                        vga::print_str(e);
                        vga::print_str(")\n");
                        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                    }
                }
            }
            _ => {
                vga::set_color(vga::Color::LightRed, vga::Color::Black);
                vga::print_str("Error: Unrecognized option '");
                vga::print_str(flag);
                vga::print_str("'. Use -m, -s, or -q.\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            }
        }
    }
}

fn parse_u32(s: &str) -> Result<u32, ()> {
    if s.is_empty() {
        return Err(());
    }
    let mut val: u32 = 0;
    for b in s.bytes() {
        if b < b'0' || b > b'9' {
            return Err(());
        }
        val = val.checked_mul(10).ok_or(())?;
        val = val.checked_add((b - b'0') as u32).ok_or(())?;
    }
    Ok(val)
}
