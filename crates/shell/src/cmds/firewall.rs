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

use keira_io::vga;
use keira_net::netfilter;

pub fn run(parts: &mut core::str::SplitWhitespace) {
    let subcmd = parts.next();
    match subcmd {
        Some("-h") | Some("--help") => unsafe {
            vga::print_str("Usage: firewall [status|enable|disable|flush]\n\n");
            vga::print_str("Description:\n  Toggle stateful Netfilter IPv4 firewall engine or flush active rules (Syscall 76).\n\n");
            vga::print_str("Options:\n  -h, --help    Show this help message and exit\n");
        },
        Some("enable") | Some("disable") | Some("toggle") => unsafe {
            let _ = netfilter::sys_netfilter(netfilter::NETFILTER_CMD_TOGGLE, 0, 0);
            vga::set_color(vga::Color::LightGreen, vga::Color::Black);
            vga::print_str("[FIREWALL] Toggled stateful Netfilter engine state [OK]\n");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        },
        Some("flush") => unsafe {
            let _ = netfilter::sys_netfilter(netfilter::NETFILTER_CMD_FLUSH, 0, 0);
            vga::set_color(vga::Color::Yellow, vga::Color::Black);
            vga::print_str("[FIREWALL] Flushed active connection tracking & rules [OK]\n");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        },
        _ => unsafe {
            let _ = netfilter::sys_netfilter(netfilter::NETFILTER_CMD_STATUS, 0, 0);
        },
    }
}
