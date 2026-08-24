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
//! Netfilter firewall state toggle and status control command.

use crate::args::CliArgs;
use keira_io::vga;
use keira_net::netfilter;

pub fn run(parts: &mut core::str::SplitWhitespace) {
    let args = CliArgs::parse(parts);

    if args.has_flag('h', "help") {
        unsafe {
            vga::set_color(vga::Color::White, vga::Color::Black);
            vga::print_str("Usage: firewall [-L] [-F] [enable|disable|toggle]\n\n");
            vga::print_str("Description:\n  Inspect or toggle the stateful IPv4 Netfilter firewall engine.\n\n");
            vga::print_str("Options:\n");
            vga::print_str(
                "  -L, --list     Display active firewall rules and connection tracking table\n",
            );
            vga::print_str(
                "  -F, --flush    Flush all connection tracking entries and custom rules\n",
            );
            vga::print_str("  -h, --help     Show this help message and exit\n");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        }
        return;
    }

    unsafe {
        let sub = args.first_positional();

        if args.has_flag('F', "flush") || sub == Some("flush") {
            let _ = netfilter::sys_netfilter(netfilter::NETFILTER_CMD_FLUSH, 0, 0);
            vga::set_color(vga::Color::Yellow, vga::Color::Black);
            vga::print_str("[FIREWALL] Flushed active connection tracking & rules [OK]\n");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            return;
        }

        if sub == Some("enable")
            || sub == Some("disable")
            || sub == Some("toggle")
            || args.has_flag('t', "toggle")
        {
            let _ = netfilter::sys_netfilter(netfilter::NETFILTER_CMD_TOGGLE, 0, 0);
            vga::set_color(vga::Color::LightGreen, vga::Color::Black);
            vga::print_str("[FIREWALL] Toggled stateful Netfilter engine state [OK]\n");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            return;
        }

        let _ = netfilter::sys_netfilter(netfilter::NETFILTER_CMD_STATUS, 0, 0);
    }
}
