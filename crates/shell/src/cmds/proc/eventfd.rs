// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

#![allow(unused_variables, unused_unsafe)]

//! Inspect and manage EventFD notification counters (Syscall 50 & 51).

use keira_io::vga;
use keira_ipc::event::eventfd::{
    close_eventfd, create_eventfd, get_eventfd_stats, get_eventfd_table, read_eventfd,
    write_eventfd, MAX_EVENTFDS,
};

pub fn run(parts: &mut core::str::SplitWhitespace) {
    let subcmd = parts.next();
    if subcmd == Some("-h") || subcmd == Some("--help") || subcmd.is_none() {
        unsafe {
            vga::set_color(vga::Color::White, vga::Color::Black);
            vga::print_str("Usage: eventfd <subcommand> [args]\n\n");
            vga::print_str(
                "Description:\n  Inspect and manage EventFD 64-bit event notification counters (Syscall 50 & 51).\n\n",
            );
            vga::print_str("Subcommands:\n");
            vga::print_str(
                "  status                         Display eventfd subsystem statistics\n",
            );
            vga::print_str(
                "  list                           List active EventFD descriptors and counters\n",
            );
            vga::print_str(
                "  create [initval]               Allocate new EventFD counter descriptor\n",
            );
            vga::print_str(
                "  read <id>                      Read and consume counter value from descriptor\n",
            );
            vga::print_str(
                "  write <id> <val>               Increment counter value on descriptor\n",
            );
            vga::print_str("  close <id>                     Close and deallocate descriptor\n");
            vga::print_str("  -h, --help                     Show this help message and exit\n");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        }
        return;
    }

    unsafe {
        match subcmd.unwrap() {
            "status" => {
                let (active, writes, reads) = get_eventfd_stats();
                vga::set_color(vga::Color::White, vga::Color::Black);
                vga::print_str("EventFD & SignalFD Subsystem Status:\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                vga::print_str("  Subsystem Engine : ");
                vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                vga::print_str("Active (In-Kernel 64-bit Counter Descriptors)\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                vga::print_str("  Active Descriptors: ");
                vga::print_u64(active as u64);
                vga::print_str(" / ");
                vga::print_u64(MAX_EVENTFDS as u64);
                vga::print_str(" allocated\n");
                vga::print_str("  Total Writes     : ");
                vga::print_u64(writes);
                vga::print_str(" ops\n");
                vga::print_str("  Total Reads      : ");
                vga::print_u64(reads);
                vga::print_str(" ops\n");
                vga::print_str(
                    "  Syscall Vectors  : Syscall 50 (eventfd) / Syscall 51 (signalfd)\n",
                );
            }
            "list" => {
                vga::set_color(vga::Color::White, vga::Color::Black);
                vga::print_str("ID   COUNTER              FLAGS\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);

                let table = get_eventfd_table();
                let mut count = 0;
                for slot in table.iter() {
                    if slot.in_use {
                        vga::print_u64(slot.id as u64);
                        vga::print_str("    ");
                        vga::print_u64(slot.count);
                        let mut digits = 1;
                        let mut temp = slot.count;
                        while temp >= 10 {
                            digits += 1;
                            temp /= 10;
                        }
                        let pad = if digits < 21 { 21 - digits } else { 1 };
                        for _ in 0..pad {
                            vga::print_str(" ");
                        }
                        vga::print_hex(slot.flags as u64);
                        vga::print_str("\n");
                        count += 1;
                    }
                }
                if count == 0 {
                    vga::print_str("  (no active EventFD descriptors)\n");
                }
            }
            "create" => {
                let initval = parts.next().and_then(|s| parse_u64(s).ok()).unwrap_or(0);
                match create_eventfd(initval, 0) {
                    Ok(id) => {
                        vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                        vga::print_str("[OK] ");
                        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                        vga::print_str("Created EventFD descriptor #");
                        vga::print_u64(id as u64);
                        vga::print_str(" (Initial Counter: ");
                        vga::print_u64(initval);
                        vga::print_str(")\n");
                    }
                    Err(e) => {
                        vga::set_color(vga::Color::LightRed, vga::Color::Black);
                        vga::print_str("Error: ");
                        vga::print_str(e);
                        vga::print_str("\n");
                        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                    }
                }
            }
            "read" => {
                let id_str = match parts.next() {
                    Some(s) => s,
                    None => {
                        vga::set_color(vga::Color::LightRed, vga::Color::Black);
                        vga::print_str("Error: Descriptor ID required. Usage: eventfd read <id>\n");
                        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                        return;
                    }
                };
                let id = match parse_u32(id_str) {
                    Ok(i) => i,
                    Err(_) => {
                        vga::set_color(vga::Color::LightRed, vga::Color::Black);
                        vga::print_str("Error: Invalid descriptor ID format\n");
                        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                        return;
                    }
                };

                match read_eventfd(id) {
                    Ok(val) => {
                        vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                        vga::print_str("[OK] ");
                        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                        vga::print_str("Read from EventFD #");
                        vga::print_u64(id as u64);
                        vga::print_str(": Counter = ");
                        vga::print_u64(val);
                        vga::print_str(" (Reset to 0)\n");
                    }
                    Err(e) => {
                        vga::set_color(vga::Color::LightRed, vga::Color::Black);
                        vga::print_str("Error: ");
                        vga::print_str(e);
                        vga::print_str("\n");
                        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                    }
                }
            }
            "write" => {
                let id_str = match parts.next() {
                    Some(s) => s,
                    None => {
                        vga::set_color(vga::Color::LightRed, vga::Color::Black);
                        vga::print_str(
                            "Error: Descriptor ID required. Usage: eventfd write <id> <val>\n",
                        );
                        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                        return;
                    }
                };
                let val_str = match parts.next() {
                    Some(s) => s,
                    None => {
                        vga::set_color(vga::Color::LightRed, vga::Color::Black);
                        vga::print_str(
                            "Error: Counter value required. Usage: eventfd write <id> <val>\n",
                        );
                        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                        return;
                    }
                };

                let id = match parse_u32(id_str) {
                    Ok(i) => i,
                    Err(_) => {
                        vga::set_color(vga::Color::LightRed, vga::Color::Black);
                        vga::print_str("Error: Invalid descriptor ID\n");
                        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                        return;
                    }
                };
                let val = match parse_u64(val_str) {
                    Ok(v) => v,
                    Err(_) => {
                        vga::set_color(vga::Color::LightRed, vga::Color::Black);
                        vga::print_str("Error: Invalid increment value\n");
                        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                        return;
                    }
                };

                match write_eventfd(id, val) {
                    Ok(_) => {
                        vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                        vga::print_str("[OK] ");
                        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                        vga::print_str("Incremented EventFD #");
                        vga::print_u64(id as u64);
                        vga::print_str(" by ");
                        vga::print_u64(val);
                        vga::print_str("\n");
                    }
                    Err(e) => {
                        vga::set_color(vga::Color::LightRed, vga::Color::Black);
                        vga::print_str("Error: ");
                        vga::print_str(e);
                        vga::print_str("\n");
                        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                    }
                }
            }
            "close" => {
                let id_str = match parts.next() {
                    Some(s) => s,
                    None => {
                        vga::set_color(vga::Color::LightRed, vga::Color::Black);
                        vga::print_str(
                            "Error: Descriptor ID required. Usage: eventfd close <id>\n",
                        );
                        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                        return;
                    }
                };
                let id = match parse_u32(id_str) {
                    Ok(i) => i,
                    Err(_) => {
                        vga::set_color(vga::Color::LightRed, vga::Color::Black);
                        vga::print_str("Error: Invalid descriptor ID\n");
                        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                        return;
                    }
                };

                match close_eventfd(id) {
                    Ok(_) => {
                        vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                        vga::print_str("[OK] ");
                        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                        vga::print_str("Closed EventFD descriptor #");
                        vga::print_u64(id as u64);
                        vga::print_str("\n");
                    }
                    Err(e) => {
                        vga::set_color(vga::Color::LightRed, vga::Color::Black);
                        vga::print_str("Error: ");
                        vga::print_str(e);
                        vga::print_str("\n");
                        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                    }
                }
            }
            unknown => {
                vga::set_color(vga::Color::LightRed, vga::Color::Black);
                vga::print_str("Error: Unrecognized subcommand '");
                vga::print_str(unknown);
                vga::print_str("'. Use 'eventfd --help' for available subcommands.\n");
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

fn parse_u64(s: &str) -> Result<u64, ()> {
    if s.is_empty() {
        return Err(());
    }
    let mut val: u64 = 0;
    for b in s.bytes() {
        if b < b'0' || b > b'9' {
            return Err(());
        }
        val = val.checked_mul(10).ok_or(())?;
        val = val.checked_add((b - b'0') as u64).ok_or(())?;
    }
    Ok(val)
}
