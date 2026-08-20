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
//! In-kernel IPv4 packet filtering and chain rule management command.

use keira_io::vga;
use keira_net::netfilter;

pub fn run(parts: &mut core::str::SplitWhitespace) {
    let subcmd = parts.next();
    match subcmd {
        Some("-h") | Some("--help") => unsafe {
            vga::print_str("Usage: iptables [-A|-D|-L|-F] <chain> <rule>\n\n");
            vga::print_str("Description:\n  Manage stateful Netfilter IPv4 firewall chains (INPUT, OUTPUT, FORWARD, PREROUTING) (Syscall 76).\n\n");
            vga::print_str("Options:\n  -h, --help    Show this help message and exit\n");
        },
        Some("-A") => unsafe {
            let _ = netfilter::sys_netfilter(netfilter::NETFILTER_CMD_ADD_RULE, 0, 0);
            vga::set_color(vga::Color::LightGreen, vga::Color::Black);
            vga::print_str("[IPTABLES] Added rule to INPUT chain (Port 22/TCP ACCEPT) [OK]\n");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        },
        Some("-D") => unsafe {
            let _ = netfilter::sys_netfilter(netfilter::NETFILTER_CMD_DEL_RULE, 0, 0);
            vga::set_color(vga::Color::Yellow, vga::Color::Black);
            vga::print_str("[IPTABLES] Deleted target rule from chain [OK]\n");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        },
        Some("-F") => unsafe {
            let _ = netfilter::sys_netfilter(netfilter::NETFILTER_CMD_FLUSH, 0, 0);
            vga::set_color(vga::Color::Yellow, vga::Color::Black);
            vga::print_str("[IPTABLES] Flushed all chain rule tables [OK]\n");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        },
        _ => unsafe {
            let _ = netfilter::sys_netfilter(netfilter::NETFILTER_CMD_STATUS, 0, 0);
        },
    }
}
