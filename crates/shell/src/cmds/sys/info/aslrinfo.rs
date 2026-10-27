// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Implementation of the 'aslrinfo' and 'aslr' shell commands.

use crate::args::CliArgs;
use keira_arch::cpu::get_entropy_features;
use keira_io::vga;
use keira_task::aslr::{
    get_aslr_limits, get_aslr_policy, set_aslr_policy, ASLR_MODE_CONSERVATIVE, ASLR_MODE_DISABLED,
    ASLR_MODE_FULL,
};
use keira_task::scheduler::TASKS;

/// Prints an integer with trailing space padding.
fn print_padded_num(val: u64, width: usize) {
    let mut digits = 1;
    let mut temp = val;
    while temp >= 10 {
        digits += 1;
        temp /= 10;
    }
    vga::print_u64(val);
    for _ in digits..width {
        vga::print_str(" ");
    }
}

/// Prints a string slice with trailing space padding.
fn print_padded_str(s: &str, width: usize) {
    vga::print_str(s);
    let len = s.len();
    for _ in len..width {
        vga::print_str(" ");
    }
}

/// Prints a formatted hexadecimal offset with column width alignment.
fn print_hex_col(val: u64, width: usize) {
    let mut buf = [0u8; 20];
    buf[0] = b'0';
    buf[1] = b'x';
    let mut len = 2;
    if val == 0 {
        buf[2] = b'0';
        len = 3;
    } else {
        let mut digits = [0u8; 16];
        let mut d_count = 0;
        let mut temp = val;
        while temp > 0 {
            let digit = (temp & 0xF) as u8;
            digits[d_count] = if digit < 10 {
                b'0' + digit
            } else {
                b'a' + (digit - 10)
            };
            d_count += 1;
            temp >>= 4;
        }
        for d in digits[..d_count].iter().rev() {
            buf[len] = *d;
            len += 1;
        }
    }
    if let Ok(s) = core::str::from_utf8(&buf[..len]) {
        vga::print_str(s);
        for _ in len..width {
            vga::print_str(" ");
        }
    }
}

/// Executes the 'aslrinfo' / 'aslr' diagnostic command.
pub fn run(parts: &mut core::str::SplitWhitespace) {
    let args = CliArgs::parse(parts);

    if args.has_flag('h', "help") {
        vga::set_color(vga::Color::White, vga::Color::Black);
        vga::print_str("Usage: aslrinfo [-v] [-h] [mode]\n\n");
        vga::print_str(
            "Description:\n  Display and configure Address Space Layout Randomization (ASLR) and hardware entropy status.\n\n",
        );
        vga::print_str("Arguments:\n");
        vga::print_str(
            "  [mode]         Set ASLR policy: 0 (disabled), 1 (conservative) or 2 (full)\n\n",
        );
        vga::print_str("Options:\n");
        vga::print_str("  -v, --verbose  Display per-task randomized memory layout table\n");
        vga::print_str("  -h, --help     Show this help message and exit\n");
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        return;
    }

    if let Some(pos) = args.first_positional() {
        if pos == "0" || pos == "1" || pos == "2" {
            let mode = if pos == "0" {
                ASLR_MODE_DISABLED
            } else if pos == "1" {
                ASLR_MODE_CONSERVATIVE
            } else {
                ASLR_MODE_FULL
            };
            set_aslr_policy(mode);
            vga::set_color(vga::Color::LightGreen, vga::Color::Black);
            vga::print_str("ASLR policy updated to mode ");
            vga::print_str(pos);
            vga::print_str("\n\n");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        } else {
            vga::set_color(vga::Color::LightRed, vga::Color::Black);
            vga::print_str("Error: Invalid ASLR mode '");
            vga::print_str(pos);
            vga::print_str("'. Expected 0, 1 or 2.\n");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            return;
        }
    }

    let policy = get_aslr_policy();
    let entropy_feat = get_entropy_features();
    let (stack_lim, mmap_lim, brk_lim) = get_aslr_limits();

    vga::set_color(vga::Color::White, vga::Color::Black);
    vga::print_str("Address Space Layout Randomization (ASLR) Subsystem:\n");

    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    vga::print_str("  Policy Mode:       ");
    match policy {
        ASLR_MODE_DISABLED => {
            vga::set_color(vga::Color::LightRed, vga::Color::Black);
            vga::print_str("[DISABLED]    ");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            vga::print_str(" Mode 0 (Deterministic layout)\n");
        }
        ASLR_MODE_CONSERVATIVE => {
            vga::set_color(vga::Color::Yellow, vga::Color::Black);
            vga::print_str("[CONSERVATIVE]");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            vga::print_str(" Mode 1 (Stack randomization only)\n");
        }
        _ => {
            vga::set_color(vga::Color::LightGreen, vga::Color::Black);
            vga::print_str("[FULL]        ");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            vga::print_str(" Mode 2 (Stack, mmap and brk randomization)\n");
        }
    }

    vga::print_str("  Hardware Entropy:  ");
    if entropy_feat.has_rdrand || entropy_feat.has_rdseed {
        vga::set_color(vga::Color::LightGreen, vga::Color::Black);
        vga::print_str("[HARDWARE]    ");
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        vga::print_str(" CPU RDRAND");
        if entropy_feat.has_rdseed {
            vga::print_str(" + RDSEED");
        }
        vga::print_str("\n");
    } else {
        vga::set_color(vga::Color::Yellow, vga::Color::Black);
        vga::print_str("[FALLBACK]    ");
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        vga::print_str(" CPU TSC/HPET SplitMix64 Mixer\n");
    }

    vga::print_str("  ProcFS Reflection: /proc/sys/kernel/randomize_va_space\n");
    vga::print_str("  Architectural Bounds:\n");
    vga::print_str("    Stack Range:     0..");
    vga::print_u64(stack_lim / 1024);
    vga::print_str(" KiB (");
    vga::print_u64(stack_lim / 4096);
    vga::print_str(" pages)\n");

    vga::print_str("    Mmap Range:      0..");
    if mmap_lim >= 1024 * 1024 {
        vga::print_u64(mmap_lim / (1024 * 1024));
        vga::print_str(" MiB (");
    } else {
        vga::print_u64(mmap_lim / 1024);
        vga::print_str(" KiB (");
    }
    vga::print_u64(mmap_lim / 4096);
    vga::print_str(" pages)\n");

    vga::print_str("    Heap (Brk) Range:0..");
    vga::print_u64(brk_lim / 1024);
    vga::print_str(" KiB (");
    vga::print_u64(brk_lim / 4096);
    vga::print_str(" pages)\n");

    if args.has_flag('v', "verbose") {
        vga::set_color(vga::Color::White, vga::Color::Black);
        vga::print_str("\n  Active Task Address Space Randomization:\n");
        vga::print_str("    PID   COMM             STACK OFFSET  MMAP OFFSET   BRK OFFSET\n");
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);

        unsafe {
            for slot in TASKS.iter() {
                if let Some(ref task) = slot {
                    vga::print_str("    ");
                    print_padded_num(task.id as u64, 5);
                    vga::print_str(" ");
                    print_padded_str(task.name, 16);
                    vga::print_str(" ");
                    print_hex_col(task.stack_aslr_offset, 13);
                    vga::print_str(" ");
                    print_hex_col(task.mmap_aslr_offset, 13);
                    vga::print_str(" ");
                    print_hex_col(task.brk_aslr_offset, 12);
                    vga::print_str("\n");
                }
            }
        }
    }
    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
}
