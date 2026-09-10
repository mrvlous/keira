// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

#![allow(unused_variables, unused_unsafe)]

//! Query and manage Fast Userspace Mutex (Futex) wait queues (Syscall 32 & 40).

use keira_io::vga;
use keira_ipc::futex::{
    futex_requeue, futex_reset, futex_wait, futex_wake, get_futex_stats, get_futex_table,
    MAX_FUTEX_WAITERS,
};

pub fn run(parts: &mut core::str::SplitWhitespace) {
    let subcmd = parts.next();
    if subcmd == Some("-h") || subcmd == Some("--help") || subcmd.is_none() {
        unsafe {
            vga::set_color(vga::Color::White, vga::Color::Black);
            vga::print_str("Usage: futex <subcommand> [args]\n\n");
            vga::print_str(
                "Description:\n  Query and manage Fast Userspace Mutex (Futex) wait queues (Syscall 32 & 40).\n\n",
            );
            vga::print_str("Subcommands:\n");
            vga::print_str("  status                         Display futex subsystem statistics\n");
            vga::print_str(
                "  list                           List active wait queue slots and blocked PIDs\n",
            );
            vga::print_str(
                "  wait <uaddr_hex> <val>         Enqueue simulated thread into wait queue\n",
            );
            vga::print_str(
                "  wake <uaddr_hex> [count]       Wake threads waiting on specified address\n",
            );
            vga::print_str(
                "  requeue <from_hex> <to_hex>    Requeue waiting threads to target address\n",
            );
            vga::print_str(
                "  reset                          Clear all active wait queues and counters\n",
            );
            vga::print_str("  -h, --help                     Show this help message and exit\n");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        }
        return;
    }

    unsafe {
        match subcmd.unwrap() {
            "status" => {
                let (waits, wakes, requeues, active) = get_futex_stats();
                vga::set_color(vga::Color::White, vga::Color::Black);
                vga::print_str("Fast Userspace Mutex (Futex) Subsystem Status:\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                vga::print_str("  Subsystem Engine : ");
                vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                vga::print_str("Active (In-Kernel Wait Queues & Hash Table)\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                vga::print_str("  Active Waiters   : ");
                vga::print_u64(active as u64);
                vga::print_str(" / ");
                vga::print_u64(MAX_FUTEX_WAITERS as u64);
                vga::print_str(" slots in use\n");
                vga::print_str("  Total Waits      : ");
                vga::print_u64(waits);
                vga::print_str(" ops\n");
                vga::print_str("  Total Wakes      : ");
                vga::print_u64(wakes);
                vga::print_str(" ops\n");
                vga::print_str("  Total Requeues   : ");
                vga::print_u64(requeues);
                vga::print_str(" ops\n");
                vga::print_str("  Syscall Vectors  : Syscall 32 (futex) / Syscall 40\n");
            }
            "list" => {
                vga::set_color(vga::Color::White, vga::Color::Black);
                vga::print_str("SLOT  UADDR        PID   EXPECTED_VAL  BITSET\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);

                let table = get_futex_table();
                let mut count = 0;
                for (idx, slot) in table.iter().enumerate() {
                    if slot.in_use {
                        vga::print_u64(idx as u64);
                        vga::print_str("     ");
                        vga::print_hex(slot.uaddr as u64);
                        vga::print_str("   ");
                        vga::print_u64(slot.pid as u64);
                        vga::print_str("     ");
                        vga::print_u64(slot.val as u64);
                        vga::print_str("             ");
                        vga::print_hex(slot.bitset as u64);
                        vga::print_str("\n");
                        count += 1;
                    }
                }
                if count == 0 {
                    vga::print_str("  (no active futex waiters in queue)\n");
                }
            }
            "wait" => {
                let uaddr_str = match parts.next() {
                    Some(s) => s,
                    None => {
                        vga::set_color(vga::Color::LightRed, vga::Color::Black);
                        vga::print_str(
                            "Error: Target address required. Usage: futex wait <uaddr_hex> <val>\n",
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
                            "Error: Expected value required. Usage: futex wait <uaddr_hex> <val>\n",
                        );
                        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                        return;
                    }
                };

                let uaddr = parse_hex_or_dec(uaddr_str);
                let val = parse_u32(val_str).unwrap_or(0);

                match futex_wait(uaddr, val, 1, 0xFFFFFFFF) {
                    Ok(_) => {
                        vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                        vga::print_str("[OK] ");
                        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                        vga::print_str("Thread (PID 1) enqueued into futex wait queue at 0x");
                        vga::print_hex(uaddr as u64);
                        vga::print_str(" (expected val: ");
                        vga::print_u64(val as u64);
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
            "wake" => {
                let uaddr_str = match parts.next() {
                    Some(s) => s,
                    None => {
                        vga::set_color(vga::Color::LightRed, vga::Color::Black);
                        vga::print_str("Error: Target address required. Usage: futex wake <uaddr_hex> [count]\n");
                        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                        return;
                    }
                };
                let count = parts.next().and_then(|s| parse_u32(s).ok()).unwrap_or(1);
                let uaddr = parse_hex_or_dec(uaddr_str);

                match futex_wake(uaddr, count, 0xFFFFFFFF) {
                    Ok(woken) => {
                        vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                        vga::print_str("[OK] ");
                        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                        vga::print_str("Woke ");
                        vga::print_u64(woken as u64);
                        vga::print_str(" waiter(s) at address 0x");
                        vga::print_hex(uaddr as u64);
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
            "requeue" => {
                let from_str = match parts.next() {
                    Some(s) => s,
                    None => {
                        vga::set_color(vga::Color::LightRed, vga::Color::Black);
                        vga::print_str("Error: Source and destination addresses required. Usage: futex requeue <from_hex> <to_hex>\n");
                        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                        return;
                    }
                };
                let to_str = match parts.next() {
                    Some(s) => s,
                    None => {
                        vga::set_color(vga::Color::LightRed, vga::Color::Black);
                        vga::print_str("Error: Destination address required. Usage: futex requeue <from_hex> <to_hex>\n");
                        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                        return;
                    }
                };
                let from_addr = parse_hex_or_dec(from_str);
                let to_addr = parse_hex_or_dec(to_str);

                match futex_requeue(from_addr, to_addr, 1) {
                    Ok(requeued) => {
                        vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                        vga::print_str("[OK] ");
                        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                        vga::print_str("Requeued ");
                        vga::print_u64(requeued as u64);
                        vga::print_str(" waiter(s) from 0x");
                        vga::print_hex(from_addr as u64);
                        vga::print_str(" to 0x");
                        vga::print_hex(to_addr as u64);
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
            "reset" => {
                futex_reset();
                vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                vga::print_str("[OK] ");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                vga::print_str("All futex wait queues and counters have been reset.\n");
            }
            unknown => {
                vga::set_color(vga::Color::LightRed, vga::Color::Black);
                vga::print_str("Error: Unrecognized subcommand '");
                vga::print_str(unknown);
                vga::print_str("'. Use 'futex --help' for available subcommands.\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            }
        }
    }
}

fn parse_hex_or_dec(s: &str) -> usize {
    if s.starts_with("0x") || s.starts_with("0X") {
        let mut val: usize = 0;
        for b in s[2..].bytes() {
            let digit = match b {
                b'0'..=b'9' => (b - b'0') as usize,
                b'a'..=b'f' => (b - b'a' + 10) as usize,
                b'A'..=b'F' => (b - b'A' + 10) as usize,
                _ => break,
            };
            val = (val << 4) | digit;
        }
        val
    } else {
        parse_u32(s).map(|v| v as usize).unwrap_or(0)
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
