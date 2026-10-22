// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Implementation of the 'pipes' supervisor shell command.

use crate::args::CliArgs;
use keira_io::vga;

/// Entry point for supervisor `pipes` command.
pub fn run(parts: &mut core::str::SplitWhitespace) {
    let args = CliArgs::parse(parts);

    if args.has_flag('h', "help") {
        vga::set_color(vga::Color::White, vga::Color::Black);
        vga::print_str("Usage: pipes [-h]\n\n");
        vga::print_str(
            "Description:\n  Display active kernel anonymous pipe channels and buffer metrics.\n\n",
        );
        vga::print_str("Options:\n  -h, --help  Show this help message and exit\n");
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        return;
    }

    unsafe {
        let table = keira_ipc::pipe::get_pipe_table();
        let (active_count, total_bytes) = keira_ipc::pipe::get_pipe_stats();

        vga::set_color(vga::Color::White, vga::Color::Black);
        vga::print_str("Active Kernel Anonymous Pipe Channels:\n\n");
        vga::print_str("  ID     STATUS   BUFFERED   READERS   WRITERS   FLAGS\n");
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);

        let mut displayed = 0usize;
        for slot in table.iter() {
            if slot.in_use {
                displayed += 1;
                vga::print_str("  ");

                // Print pipe ID
                print_num(slot.id as usize);
                for _ in 0..7usize.saturating_sub(num_len(slot.id as usize)) {
                    vga::print_str(" ");
                }

                vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                vga::print_str("ACTIVE   ");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);

                // Print buffered bytes
                print_num(slot.count);
                vga::print_str(" B");
                for _ in 0..9usize.saturating_sub(num_len(slot.count) + 2) {
                    vga::print_str(" ");
                }

                // Readers count
                print_num(slot.readers_count as usize);
                for _ in 0..10usize.saturating_sub(num_len(slot.readers_count as usize)) {
                    vga::print_str(" ");
                }

                // Writers count
                print_num(slot.writers_count as usize);
                for _ in 0..10usize.saturating_sub(num_len(slot.writers_count as usize)) {
                    vga::print_str(" ");
                }

                if (slot.flags & keira_ipc::pipe::O_NONBLOCK) != 0 {
                    vga::print_str("O_NONBLOCK");
                } else {
                    vga::print_str("BLOCKING");
                }
                vga::print_str("\n");
            }
        }

        if displayed == 0 {
            vga::print_str("  (no active anonymous pipes allocated)\n");
        }

        vga::print_str("\n");
        vga::set_color(vga::Color::LightGreen, vga::Color::Black);
        vga::print_str("Summary: ");
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        print_num(active_count);
        vga::print_str(" active channels, ");
        print_num(total_bytes);
        vga::print_str(" bytes currently buffered.\n");
    }
}

fn num_len(mut val: usize) -> usize {
    if val == 0 {
        return 1;
    }
    let mut len = 0;
    while val > 0 {
        len += 1;
        val /= 10;
    }
    len
}

fn print_num(val: usize) {
    let mut buf = [0u8; 20];
    let mut i = 0;
    let mut temp = val;
    if temp == 0 {
        vga::print_str("0");
        return;
    }
    while temp > 0 {
        buf[i] = b'0' + (temp % 10) as u8;
        temp /= 10;
        i += 1;
    }
    while i > 0 {
        i -= 1;
        let s = core::str::from_utf8(&buf[i..i + 1]).unwrap_or("0");
        vga::print_str(s);
    }
}
