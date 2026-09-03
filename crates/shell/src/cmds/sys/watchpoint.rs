// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Hardware debug register (DR0-DR7) watchpoint control command.

#![allow(unused_unsafe)]

use crate::args::CliArgs;
use keira_arch::debug::hw_breakpoint::{
    clear_watchpoint, read_dr0, read_dr1, read_dr2, read_dr3, read_dr6, read_dr7, set_watchpoint,
    WatchpointCondition, WatchpointSize, WATCHPOINTS,
};
use keira_io::vga;

fn parse_hex(s: &str) -> Option<usize> {
    let s = s.trim_start_matches("0x").trim_start_matches("0X");
    let mut val = 0usize;
    if s.is_empty() {
        return None;
    }
    for b in s.bytes() {
        val = val.checked_mul(16)?;
        match b {
            b'0'..=b'9' => val = val.checked_add((b - b'0') as usize)?,
            b'a'..=b'f' => val = val.checked_add((b - b'a' + 10) as usize)?,
            b'A'..=b'F' => val = val.checked_add((b - b'A' + 10) as usize)?,
            _ => return None,
        }
    }
    Some(val)
}

/// Entry point for `watchpoint` command.
pub fn run(parts: &mut core::str::SplitWhitespace) {
    let args = CliArgs::parse(parts);

    if args.has_flag('h', "help") {
        unsafe {
            vga::set_color(vga::Color::White, vga::Color::Black);
            vga::print_str("Usage: watchpoint [list | set <slot> <addr> [w|rw|x] [1|2|4|8] | clear <slot>]\n\n");
            vga::print_str("Description:\n");
            vga::print_str(
                "  Control x86 hardware debug registers (DR0-DR7) and memory watchpoints.\n\n",
            );
            vga::print_str("Subcommands:\n");
            vga::print_str(
                "  list            Display active watchpoints and DR6/DR7 register status\n",
            );
            vga::print_str("  set <slot> <addr> [cond] [len]  Set watchpoint on slot 0..3\n");
            vga::print_str("  clear <slot>    Clear and disable watchpoint on slot 0..3\n");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        }
        return;
    }

    let subcmd = args.positional(0).unwrap_or("list");

    match subcmd {
        "set" => {
            let slot_str = match args.positional(1) {
                Some(s) => s,
                None => {
                    unsafe {
                        vga::set_color(vga::Color::LightRed, vga::Color::Black);
                        vga::print_str("Error: missing watchpoint slot (0..3)\n");
                        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                    }
                    return;
                }
            };

            let slot: usize = match slot_str.parse() {
                Ok(s) if s < 4 => s,
                _ => {
                    unsafe {
                        vga::set_color(vga::Color::LightRed, vga::Color::Black);
                        vga::print_str("Error: slot must be an integer between 0 and 3\n");
                        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                    }
                    return;
                }
            };

            let addr_str = match args.positional(2) {
                Some(s) => s,
                None => {
                    unsafe {
                        vga::set_color(vga::Color::LightRed, vga::Color::Black);
                        vga::print_str(
                            "Error: missing linear memory address (hex format: 0x...)\n",
                        );
                        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                    }
                    return;
                }
            };

            let addr = match parse_hex(addr_str) {
                Some(a) => a,
                None => {
                    unsafe {
                        vga::set_color(vga::Color::LightRed, vga::Color::Black);
                        vga::print_str("Error: invalid hexadecimal address format\n");
                        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                    }
                    return;
                }
            };

            let cond = match args.positional(3).unwrap_or("w") {
                "x" | "exec" => WatchpointCondition::Execution,
                "rw" | "readwrite" => WatchpointCondition::DataReadWrite,
                _ => WatchpointCondition::DataWrite,
            };

            let size = match args.positional(4).unwrap_or("4") {
                "1" => WatchpointSize::Byte1,
                "2" => WatchpointSize::Byte2,
                "8" => WatchpointSize::Byte8,
                _ => WatchpointSize::Byte4,
            };

            match set_watchpoint(slot, addr, cond, size) {
                Ok(()) => unsafe {
                    vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                    vga::print_str("[OK] Hardware watchpoint set on slot ");
                    vga::print_u64(slot as u64);
                    vga::print_str(" at address ");
                    vga::print_hex(addr as u64);
                    vga::print_str("\n");
                    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                },
                Err(e) => unsafe {
                    vga::set_color(vga::Color::LightRed, vga::Color::Black);
                    vga::print_str("Error: ");
                    vga::print_str(e);
                    vga::print_str("\n");
                    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                },
            }
        }
        "clear" => {
            let slot_str = match args.positional(1) {
                Some(s) => s,
                None => {
                    unsafe {
                        vga::set_color(vga::Color::LightRed, vga::Color::Black);
                        vga::print_str("Error: missing watchpoint slot (0..3)\n");
                        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                    }
                    return;
                }
            };

            let slot: usize = match slot_str.parse() {
                Ok(s) if s < 4 => s,
                _ => {
                    unsafe {
                        vga::set_color(vga::Color::LightRed, vga::Color::Black);
                        vga::print_str("Error: slot must be an integer between 0 and 3\n");
                        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                    }
                    return;
                }
            };

            match clear_watchpoint(slot) {
                Ok(()) => unsafe {
                    vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                    vga::print_str("[OK] Hardware watchpoint cleared on slot ");
                    vga::print_u64(slot as u64);
                    vga::print_str("\n");
                    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                },
                Err(e) => unsafe {
                    vga::set_color(vga::Color::LightRed, vga::Color::Black);
                    vga::print_str("Error: ");
                    vga::print_str(e);
                    vga::print_str("\n");
                    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                },
            }
        }
        _ => unsafe {
            vga::set_color(vga::Color::Yellow, vga::Color::Black);
            vga::print_str("x86 Hardware Debug Registers & Watchpoints:\n");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);

            vga::print_str("  DR6 (Status)  : ");
            vga::print_hex(read_dr6() as u64);
            vga::print_str("   DR7 (Control) : ");
            vga::print_hex(read_dr7() as u64);
            vga::print_str("\n\n");

            vga::set_color(vga::Color::White, vga::Color::Black);
            vga::print_str("SLOT  REGISTER  ADDRESS             CONDITION       SIZE    STATUS\n");
            vga::set_color(vga::Color::DarkGrey, vga::Color::Black);
            vga::print_str(
                "----  --------  ------------------  --------------  ------  --------\n",
            );
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);

            for slot in 0..4 {
                vga::print_str("DR");
                vga::print_u64(slot as u64);
                vga::print_str("   DR");
                vga::print_u64(slot as u64);
                vga::print_str("       ");

                let addr = match slot {
                    0 => read_dr0(),
                    1 => read_dr1(),
                    2 => read_dr2(),
                    3 => read_dr3(),
                    _ => 0,
                };
                vga::print_hex(addr as u64);
                vga::print_str("  ");

                if let Some(entry) = WATCHPOINTS[slot] {
                    match entry.condition {
                        WatchpointCondition::Execution => vga::print_str("Execution (x)   "),
                        WatchpointCondition::DataWrite => vga::print_str("Data Write (w)  "),
                        WatchpointCondition::IoReadWrite => vga::print_str("I/O Port (io)   "),
                        WatchpointCondition::DataReadWrite => vga::print_str("Read/Write (rw) "),
                    }

                    match entry.size {
                        WatchpointSize::Byte1 => vga::print_str("1-byte  "),
                        WatchpointSize::Byte2 => vga::print_str("2-byte  "),
                        WatchpointSize::Byte8 => vga::print_str("8-byte  "),
                        WatchpointSize::Byte4 => vga::print_str("4-byte  "),
                    }

                    vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                    vga::print_str("ACTIVE\n");
                    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                } else {
                    vga::print_str("Disabled        -       ");
                    vga::set_color(vga::Color::DarkGrey, vga::Color::Black);
                    vga::print_str("INACTIVE\n");
                    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                }
            }
        },
    }
}
