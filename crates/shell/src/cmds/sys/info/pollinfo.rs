// SPDX-License-Identifier: GPL-2.0-only
/*
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra <mrvlous@proton.me>
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 */

//! Implementation of the 'pollinfo' and 'poll' supervisor shell commands.

use crate::args::CliArgs;
use keira_io::vga;

fn print_label_and_u64(label: &str, val: u64, suffix: &str) {
    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    vga::print_str("  ");
    vga::print_str(label);
    let pad = 30usize.saturating_sub(label.len());
    for _ in 0..pad {
        vga::print_str(" ");
    }
    vga::print_str(": ");
    vga::set_color(vga::Color::White, vga::Color::Black);
    vga::print_u64(val);
    if !suffix.is_empty() {
        vga::print_str(" ");
        vga::print_str(suffix);
    }
    vga::print_str("\n");
}

/// Executes the 'pollinfo' / 'poll' supervisor diagnostic command.
pub fn run(parts: &mut core::str::SplitWhitespace) {
    let args = CliArgs::parse(parts);

    if args.has_flag('h', "help") {
        vga::set_color(vga::Color::White, vga::Color::Black);
        vga::print_str("Usage: pollinfo [-h]\n\n");
        vga::print_str(
            "Description:\n  Display POSIX I/O multiplexing telemetry and sys_poll invocation metrics.\n\n",
        );
        vga::print_str("Options:\n");
        vga::print_str("  -h, --help  Show this help message and exit\n");
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        return;
    }

    let stats = keira_syscall::get_poll_telemetry();

    vga::set_color(vga::Color::LightGreen, vga::Color::Black);
    vga::print_str("=== POSIX I/O Multiplexing Subsystem (SYS_POLL) ===\n\n");
    vga::set_color(vga::Color::White, vga::Color::Black);

    print_label_and_u64("Total sys_poll Invocations", stats.total_polls, "calls");
    print_label_and_u64(
        "Total Ready Event Signals",
        stats.total_ready_events,
        "events",
    );
    print_label_and_u64("Timeout Expirations", stats.total_timeouts, "events");
    print_label_and_u64("Hangup (POLLHUP) Events", stats.total_hup_events, "events");
    print_label_and_u64("Invalid FD (POLLNVAL)", stats.total_nval_events, "events");
    print_label_and_u64("Max Monitored Descriptors", 64, "descriptors / call");

    vga::print_str("\n");
    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    vga::print_str(
        "  Supported Channels        : Anonymous Pipes, UNIX Sockets, TCP/UDP, TTY & VFS\n",
    );
    vga::print_str("  Standard Compliance       : POSIX.1-2008 / Single UNIX Specification v4\n");
}
