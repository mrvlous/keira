// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! In-kernel IPv4 packet filtering and chain rule management command.

use crate::args::CliArgs;
use keira_io::vga;
use keira_net::netfilter;

fn parse_u16(s: &str) -> Option<u16> {
    let mut val: u16 = 0;
    if s.is_empty() {
        return None;
    }
    for &b in s.as_bytes() {
        if b.is_ascii_digit() {
            val = val.checked_mul(10)?.checked_add((b - b'0') as u16)?;
        } else {
            return None;
        }
    }
    Some(val)
}

pub fn run(parts: &mut core::str::SplitWhitespace) {
    let args = CliArgs::parse(parts);

    if args.has_flag('h', "help") {
        vga::set_color(vga::Color::White, vga::Color::Black);
        vga::print_str("Usage: iptables [-L] [-A <chain> [-p <proto>] [--dport <port>] -j <action>] [-D <rule_num>] [-F]\n\n");
        vga::print_str("Description:\n  Manage stateful Netfilter IPv4 firewall chains and dynamic rule tables.\n\n");
        vga::print_str("Options & Subcommands:\n");
        vga::print_str("  -L, --list     List all active rules in firewall chains (default)\n");
        vga::print_str("  -A, --append   Append a rule to target chain (e.g. iptables -A INPUT -p tcp --dport 8080 -j DROP)\n");
        vga::print_str("  -D, --delete   Delete a rule by 1-indexed number (e.g. iptables -D 4)\n");
        vga::print_str("  -F, --flush    Flush all chain rule tables and reset match counters\n");
        vga::print_str("  -h, --help     Show this help message and exit\n");
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        return;
    }

    if args.has_flag('F', "flush") || args.first_positional() == Some("flush") {
        netfilter::flush_rules();
        vga::set_color(vga::Color::Yellow, vga::Color::Black);
        vga::print_str("[IPTABLES] Flushed all chain rule tables [OK]\n");
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        return;
    }

    if args.has_flag('D', "delete") || args.first_positional() == Some("del") {
        let num_str = args
            .get_opt('D', "delete")
            .or_else(|| args.positional(1))
            .unwrap_or("1");

        if let Some(idx) = parse_u16(num_str) {
            match netfilter::delete_rule(idx as usize) {
                Ok(_) => {
                    vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                    vga::print_str("[IPTABLES] Deleted rule ");
                    vga::print_u64(idx as u64);
                    vga::print_str(" from chain [OK]\n");
                    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                }
                Err(e) => {
                    vga::set_color(vga::Color::LightRed, vga::Color::Black);
                    vga::print_str("[FAILED] ");
                    vga::print_str(e);
                    vga::print_str("\n");
                    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                }
            }
        } else {
            vga::set_color(vga::Color::LightRed, vga::Color::Black);
            vga::print_str("[FAILED] Invalid rule number. Specify a numeric index (1..16)\n");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        }
        return;
    }

    if args.has_flag('A', "append") || args.first_positional() == Some("add") {
        let chain = args
            .get_opt('A', "append")
            .or_else(|| args.positional(1))
            .unwrap_or("INPUT");

        let proto = args
            .get_opt('p', "proto")
            .or_else(|| args.positional(2))
            .unwrap_or("TCP");

        let dport = args
            .get_opt('d', "dport")
            .and_then(parse_u16)
            .or_else(|| args.positional(3).and_then(parse_u16))
            .unwrap_or(0);

        let action = args
            .get_opt('j', "jump")
            .or_else(|| args.positional(4))
            .unwrap_or("ACCEPT");

        match netfilter::add_rule(chain, proto, dport, action, "0.0.0.0/0", "0.0.0.0/0") {
            Ok(rule_num) => {
                vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                vga::print_str("[IPTABLES] Added rule ");
                vga::print_u64(rule_num as u64);
                vga::print_str(" to ");
                vga::print_str(chain);
                vga::print_str(" (Port ");
                vga::print_u64(dport as u64);
                vga::print_str("/");
                vga::print_str(proto);
                vga::print_str(" ");
                vga::print_str(action);
                vga::print_str(") [OK]\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            }
            Err(e) => {
                vga::set_color(vga::Color::LightRed, vga::Color::Black);
                vga::print_str("[FAILED] ");
                vga::print_str(e);
                vga::print_str("\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            }
        }
        return;
    }

    // Default: display rule list
    unsafe {
        netfilter::print_firewall_status();
    }
}
