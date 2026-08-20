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

use crate::state::*;
use keira_io::vga;

pub fn run(parts: &mut core::str::SplitWhitespace) {
    if let Some("-h") | Some("--help") = parts.next() {
        unsafe {
            vga::print_str("Usage: history\n\n");
            vga::print_str(
                "Description:\n  Print the ring buffer of recently entered shell commands.\n\n",
            );
            vga::print_str("Options:\n  -h, --help    Show this help message and exit\n");
        }
        return;
    }

    unsafe {
        if HISTORY_COUNT == 0 {
            vga::print_str("No history entries yet.\n");
            return;
        }

        // Print up to HISTORY_SIZE recent commands
        let limit = if HISTORY_COUNT < HISTORY_SIZE {
            HISTORY_COUNT
        } else {
            HISTORY_SIZE
        };

        // Determine starting offset in the ring buffer
        let start_idx = if HISTORY_COUNT < HISTORY_SIZE {
            0
        } else {
            HISTORY_COUNT % HISTORY_SIZE
        };

        vga::print_str("Command History:\n");
        for i in 0..limit {
            let idx = (start_idx + i) % HISTORY_SIZE;
            vga::print_str(" ");
            vga::print_u64((HISTORY_COUNT - limit + i + 1) as u64);
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
