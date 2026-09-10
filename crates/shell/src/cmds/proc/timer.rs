// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

#![allow(unused_variables, unused_unsafe)]

//! Query and manage POSIX High-Resolution Timers (Syscall 45 & 46).

use keira_arch::timer::{
    cancel_timer, create_timer, get_timer_stats, get_timer_table, CLOCK_MONOTONIC, CLOCK_REALTIME,
    MAX_POSIX_TIMERS,
};
use keira_io::vga;

pub fn run(parts: &mut core::str::SplitWhitespace) {
    let subcmd = parts.next();
    if subcmd == Some("-h") || subcmd == Some("--help") || subcmd.is_none() {
        unsafe {
            vga::set_color(vga::Color::White, vga::Color::Black);
            vga::print_str("Usage: timer <subcommand> [args]\n\n");
            vga::print_str(
                "Description:\n  Query and manage POSIX High-Resolution Interval Timers (Syscall 45 & 46).\n\n",
            );
            vga::print_str("Subcommands:\n");
            vga::print_str("  status                         Display timer subsystem statistics and clock source\n");
            vga::print_str(
                "  list                           List active interval timers and overrun stats\n",
            );
            vga::print_str("  create <interval_ms> [clock]   Create new POSIX interval timer (default: monotonic)\n");
            vga::print_str("  cancel <id>                    Cancel and delete interval timer\n");
            vga::print_str("  -h, --help                     Show this help message and exit\n");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        }
        return;
    }

    unsafe {
        match subcmd.unwrap() {
            "status" => {
                let (active, expirations) = get_timer_stats();
                vga::set_color(vga::Color::White, vga::Color::Black);
                vga::print_str("POSIX High-Resolution Timer Subsystem Status:\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                vga::print_str("  Subsystem Engine : ");
                vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                vga::print_str("Active (Hardware APIC / LAPIC Tick Driver)\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                vga::print_str("  Clock Sources    : CLOCK_MONOTONIC (1), CLOCK_REALTIME (0)\n");
                vga::print_str("  Active Timers    : ");
                vga::print_u64(active as u64);
                vga::print_str(" / ");
                vga::print_u64(MAX_POSIX_TIMERS as u64);
                vga::print_str(" allocated\n");
                vga::print_str("  Total Expirations: ");
                vga::print_u64(expirations);
                vga::print_str(" interrupts serviced\n");
                vga::print_str(
                    "  Syscall Vectors  : Syscall 45 (timer_create) / Syscall 46 (timer_settime)\n",
                );
            }
            "list" => {
                vga::set_color(vga::Color::White, vga::Color::Black);
                vga::print_str("ID   CLOCK        INTERVAL_MS  OVERRUNS  STATUS\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);

                let table = get_timer_table();
                let mut count = 0;
                for slot in table.iter() {
                    if slot.active {
                        vga::print_u64(slot.timer_id);
                        vga::print_str("    ");
                        if slot.clock_id == CLOCK_MONOTONIC {
                            vga::print_str("MONOTONIC    ");
                        } else {
                            vga::print_str("REALTIME     ");
                        }
                        vga::print_u64(slot.interval_ms);
                        vga::print_str(" ms        ");
                        vga::print_u64(slot.overruns);
                        vga::print_str("         ");
                        vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                        vga::print_str("ARMED\n");
                        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                        count += 1;
                    }
                }
                if count == 0 {
                    vga::print_str("  (no active interval timers)\n");
                }
            }
            "create" => {
                let ms_str = match parts.next() {
                    Some(s) => s,
                    None => {
                        vga::set_color(vga::Color::LightRed, vga::Color::Black);
                        vga::print_str(
                            "Error: Interval in milliseconds required. Usage: timer create <ms>\n",
                        );
                        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                        return;
                    }
                };
                let ms = match parse_u64(ms_str) {
                    Ok(v) if v > 0 => v,
                    _ => {
                        vga::set_color(vga::Color::LightRed, vga::Color::Black);
                        vga::print_str("Error: Interval must be a positive integer\n");
                        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                        return;
                    }
                };

                let clock = match parts.next() {
                    Some("realtime") | Some("0") => CLOCK_REALTIME,
                    _ => CLOCK_MONOTONIC,
                };

                match create_timer(clock, ms) {
                    Ok(id) => {
                        vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                        vga::print_str("[OK] ");
                        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                        vga::print_str("Created POSIX timer #");
                        vga::print_u64(id);
                        vga::print_str(" (Interval: ");
                        vga::print_u64(ms);
                        vga::print_str(" ms, Clock: ");
                        vga::print_str(if clock == CLOCK_MONOTONIC {
                            "MONOTONIC"
                        } else {
                            "REALTIME"
                        });
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
            "cancel" => {
                let id_str = match parts.next() {
                    Some(s) => s,
                    None => {
                        vga::set_color(vga::Color::LightRed, vga::Color::Black);
                        vga::print_str("Error: Timer ID required. Usage: timer cancel <id>\n");
                        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                        return;
                    }
                };
                let id = match parse_u64(id_str) {
                    Ok(v) => v,
                    Err(_) => {
                        vga::set_color(vga::Color::LightRed, vga::Color::Black);
                        vga::print_str("Error: Invalid timer ID\n");
                        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                        return;
                    }
                };

                match cancel_timer(id) {
                    Ok(_) => {
                        vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                        vga::print_str("[OK] ");
                        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                        vga::print_str("Canceled POSIX timer #");
                        vga::print_u64(id);
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
                vga::print_str("'. Use 'timer --help' for available subcommands.\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            }
        }
    }
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
