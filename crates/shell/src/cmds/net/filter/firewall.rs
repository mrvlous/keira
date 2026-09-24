// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Netfilter firewall state toggle, flush, and status control command.

use crate::args::CliArgs;
use keira_io::vga;
use keira_net::netfilter;

pub fn run(parts: &mut core::str::SplitWhitespace) {
    let args = CliArgs::parse(parts);

    if args.has_flag('h', "help") {
        vga::set_color(vga::Color::White, vga::Color::Black);
        vga::print_str("Usage: firewall [status | enable | disable | toggle | flush]\n\n");
        vga::print_str("Description:\n  Inspect, enable, disable, or flush the stateful IPv4 Netfilter firewall engine.\n\n");
        vga::print_str("Options & Subcommands:\n");
        vga::print_str(
            "  status         Display packet counters and active chain rules (default)\n",
        );
        vga::print_str("  enable         Activate packet inspection and filtering\n");
        vga::print_str("  disable        Bypass packet inspection (pass-through mode)\n");
        vga::print_str("  toggle         Toggle between enabled and disabled states\n");
        vga::print_str("  flush, -F      Flush all custom rules and reset match counters\n");
        vga::print_str("  -h, --help     Show this help message and exit\n");
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        return;
    }

    unsafe {
        let sub = args.first_positional().unwrap_or("status");

        if args.has_flag('F', "flush") || sub == "flush" {
            let _ = netfilter::sys_netfilter(netfilter::NETFILTER_CMD_FLUSH, 0, 0);
            vga::set_color(vga::Color::Yellow, vga::Color::Black);
            vga::print_str("[FIREWALL] Flushed active connection tracking & rules [OK]\n");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            return;
        }

        if sub == "enable" {
            netfilter::NETFILTER_ENABLED = true;
            vga::set_color(vga::Color::LightGreen, vga::Color::Black);
            vga::print_str("[FIREWALL] Enabled stateful Netfilter engine [OK]\n");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            return;
        }

        if sub == "disable" {
            netfilter::NETFILTER_ENABLED = false;
            vga::set_color(vga::Color::Yellow, vga::Color::Black);
            vga::print_str("[FIREWALL] Disabled stateful Netfilter engine (Bypass mode) [OK]\n");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            return;
        }

        if sub == "toggle" || args.has_flag('t', "toggle") {
            let _ = netfilter::sys_netfilter(netfilter::NETFILTER_CMD_TOGGLE, 0, 0);
            vga::set_color(vga::Color::LightGreen, vga::Color::Black);
            vga::print_str("[FIREWALL] Toggled stateful Netfilter engine state [OK]\n");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            return;
        }

        let _ = netfilter::sys_netfilter(netfilter::NETFILTER_CMD_STATUS, 0, 0);
    }
}
