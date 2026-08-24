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
//! Prints the ring buffer of recently entered commands.

use crate::args::CliArgs;
use crate::state::*;
use keira_io::vga;

pub fn run(parts: &mut core::str::SplitWhitespace) {
    let args = CliArgs::parse(parts);

    if args.has_flag('h', "help") {
        unsafe {
            vga::set_color(vga::Color::White, vga::Color::Black);
            vga::print_str("Usage: history [-c] [-n <count>]\n\n");
            vga::print_str("Description:\n  Print or clear the ring buffer of recently entered shell commands.\n\n");
            vga::print_str("Options:\n");
            vga::print_str("  -c, --clear    Clear all history entries from memory\n");
            vga::print_str("  -n, --limit    Show only the most recent N history entries\n");
            vga::print_str("  -h, --help     Show this help message and exit\n");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        }
        return;
    }

    unsafe {
        if args.has_flag('c', "clear") {
            HISTORY_COUNT = 0;
            HISTORY_INDEX = 0;
            vga::set_color(vga::Color::LightGreen, vga::Color::Black);
            vga::print_str("[OK] Shell command history cleared\n");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            return;
        }

        if HISTORY_COUNT == 0 {
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            vga::print_str("No history entries yet.\n");
            return;
        }

        let mut desired_limit = HISTORY_SIZE;
        if let Some(opt_val) = args.get_opt('n', "limit") {
            if let Ok(num) = opt_val.parse::<usize>() {
                desired_limit = num;
            }
        } else if let Some(pos) = args.first_positional() {
            if let Ok(num) = pos.parse::<usize>() {
                desired_limit = num;
            }
        }

        let max_available = if HISTORY_COUNT < HISTORY_SIZE {
            HISTORY_COUNT
        } else {
            HISTORY_SIZE
        };

        let limit = if desired_limit < max_available {
            desired_limit
        } else {
            max_available
        };

        let start_idx = if HISTORY_COUNT < HISTORY_SIZE {
            HISTORY_COUNT - limit
        } else {
            (HISTORY_COUNT % HISTORY_SIZE + (HISTORY_SIZE - limit)) % HISTORY_SIZE
        };

        vga::set_color(vga::Color::White, vga::Color::Black);
        vga::print_str("Command History:\n");
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);

        for i in 0..limit {
            let idx = (start_idx + i) % HISTORY_SIZE;
            vga::print_str(" ");
            let entry_num = HISTORY_COUNT - limit + i + 1;
            vga::print_u64(entry_num as u64);
            vga::print_str("  ");

            let len = HISTORY_LENS[idx];
            let cmd_slice = &HISTORY[idx][..len];
            if let Ok(s) = core::str::from_utf8(cmd_slice) {
                vga::print_str(s);
            }
            vga::print_str("\n");
        }
    }
}
