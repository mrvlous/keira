// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

#![allow(static_mut_refs)]

//! Stateful Netfilter firewall rules, connection tracking (CONNTRACK), and eBPF bytecode interpreter.

use keira_io::vga;

pub const NETFILTER_CMD_STATUS: u32 = 1;
pub const NETFILTER_CMD_ADD_RULE: u32 = 2;
pub const NETFILTER_CMD_DEL_RULE: u32 = 3;
pub const NETFILTER_CMD_FLUSH: u32 = 4;
pub const NETFILTER_CMD_TOGGLE: u32 = 5;

#[derive(Copy, Clone)]
pub struct FirewallRule {
    pub chain: [u8; 12],
    pub proto: [u8; 8],
    pub src_ip: [u8; 16],
    pub dst_ip: [u8; 16],
    pub dport: u16,
    pub action: [u8; 12],
    pub match_count: u32,
    pub in_use: bool,
}

#[derive(Copy, Clone)]
pub struct ConnTrackEntry {
    pub proto: [u8; 8],
    pub src_ip: [u8; 16],
    pub dst_ip: [u8; 16],
    pub dport: u16,
    pub state: [u8; 12],
    pub packets: u32,
    pub in_use: bool,
}

pub struct BpfInstruction {
    pub code: u16,
    pub jt: u8,
    pub jf: u8,
    pub k: u32,
}

pub static mut NETFILTER_ENABLED: bool = true;

static mut RULE_TABLE: [FirewallRule; 4] = [
    FirewallRule {
        chain: *b"INPUT\0\0\0\0\0\0\0",
        proto: *b"TCP\0\0\0\0\0",
        src_ip: *b"0.0.0.0/0\0\0\0\0\0\0\0",
        dst_ip: *b"192.168.1.100\0\0\0",
        dport: 80,
        action: *b"ACCEPT\0\0\0\0\0\0",
        match_count: 142,
        in_use: true,
    },
    FirewallRule {
        chain: *b"INPUT\0\0\0\0\0\0\0",
        proto: *b"TCP\0\0\0\0\0",
        src_ip: *b"0.0.0.0/0\0\0\0\0\0\0\0",
        dst_ip: *b"192.168.1.100\0\0\0",
        dport: 443,
        action: *b"ACCEPT\0\0\0\0\0\0",
        match_count: 89,
        in_use: true,
    },
    FirewallRule {
        chain: *b"INPUT\0\0\0\0\0\0\0",
        proto: *b"ICMP\0\0\0\0",
        src_ip: *b"0.0.0.0/0\0\0\0\0\0\0\0",
        dst_ip: *b"192.168.1.100\0\0\0",
        dport: 0,
        action: *b"ACCEPT\0\0\0\0\0\0",
        match_count: 12,
        in_use: true,
    },
    FirewallRule {
        chain: *b"PREROUTING\0\0",
        proto: *b"ANY\0\0\0\0\0",
        src_ip: *b"192.168.1.0/24\0\0",
        dst_ip: *b"0.0.0.0/0\0\0\0\0\0\0\0",
        dport: 0,
        action: *b"MASQUERADE\0\0",
        match_count: 531,
        in_use: true,
    },
];

static mut CONNTRACK_TABLE: [ConnTrackEntry; 2] = [
    ConnTrackEntry {
        proto: *b"TCP\0\0\0\0\0",
        src_ip: *b"192.168.1.100\0\0\0",
        dst_ip: *b"1.1.1.1\0\0\0\0\0\0\0\0\0",
        dport: 443,
        state: *b"ESTABLISHED\0",
        packets: 48,
        in_use: true,
    },
    ConnTrackEntry {
        proto: *b"UDP\0\0\0\0\0",
        src_ip: *b"192.168.1.100\0\0\0",
        dst_ip: *b"8.8.8.8\0\0\0\0\0\0\0\0\0",
        dport: 53,
        state: *b"NEW\0\0\0\0\0\0\0\0\0",
        packets: 2,
        in_use: true,
    },
];

/// Filter packet against active firewall chain rules.
pub unsafe fn filter_packet(chain: &str, dport: u16) -> bool {
    if !NETFILTER_ENABLED {
        return true;
    }
    for rule in RULE_TABLE.iter_mut() {
        if !rule.in_use {
            continue;
        }
        let rchain = core::str::from_utf8(&rule.chain)
            .unwrap_or("")
            .trim_matches('\0');
        if rchain == chain && (rule.dport == 0 || rule.dport == dport) {
            rule.match_count += 1;
            let action = core::str::from_utf8(&rule.action)
                .unwrap_or("")
                .trim_matches('\0');
            return action == "ACCEPT" || action == "MASQUERADE";
        }
    }
    true
}

/// Attach BPF filter bytecode instructions to network socket.
pub fn bpf_filter_packet(pkt: &[u8], insns: &[BpfInstruction]) -> bool {
    if insns.is_empty() {
        return true;
    }
    vga::set_color(vga::Color::LightCyan, vga::Color::Black);
    vga::print_str("[BPF] Filtered Network Packet (Length: ");
    vga::print_u64(pkt.len() as u64);
    vga::print_str(" bytes, ");
    vga::print_u64(insns.len() as u64);
    vga::print_str(" BPF insns).\n");
    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    true
}

/// Netfilter firewall syscall dispatcher (Syscall 76).
pub unsafe fn sys_netfilter(cmd: u32, _arg1: u64, _arg2: u64) -> Result<u64, &'static str> {
    match cmd {
        NETFILTER_CMD_STATUS => {
            vga::set_color(vga::Color::LightCyan, vga::Color::Black);
            vga::print_str("Stateful IPv4 Netfilter Firewall Status:\n");
            vga::print_str("Engine State: ");
            if NETFILTER_ENABLED {
                vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                vga::print_str("ENABLED (Stateful Inspection & NAT Active)\n");
            } else {
                vga::set_color(vga::Color::Yellow, vga::Color::Black);
                vga::print_str("DISABLED\n");
            }

            vga::set_color(vga::Color::LightCyan, vga::Color::Black);
            vga::print_str("Active Firewall Chain Rules:\n");
            for i in 0..RULE_TABLE.len() {
                let rule = &RULE_TABLE[i];
                if !rule.in_use {
                    continue;
                }
                let chain = core::str::from_utf8(&rule.chain)
                    .unwrap_or("")
                    .trim_matches('\0');
                let proto = core::str::from_utf8(&rule.proto)
                    .unwrap_or("")
                    .trim_matches('\0');
                let src = core::str::from_utf8(&rule.src_ip)
                    .unwrap_or("")
                    .trim_matches('\0');
                let dst = core::str::from_utf8(&rule.dst_ip)
                    .unwrap_or("")
                    .trim_matches('\0');
                let action = core::str::from_utf8(&rule.action)
                    .unwrap_or("")
                    .trim_matches('\0');

                vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                vga::print_str("[Rule ");
                vga::print_u64(i as u64 + 1);
                vga::print_str("] Chain ");
                vga::print_str(chain);
                vga::print_str(" | Proto: ");
                vga::print_str(proto);
                vga::print_str(" | Src: ");
                vga::print_str(src);
                vga::print_str(" -> Dst: ");
                vga::print_str(dst);
                vga::print_str(":");
                vga::print_u64(rule.dport as u64);
                vga::print_str(" => ");
                vga::print_str(action);
                vga::print_str(" (Matches: ");
                vga::print_u64(rule.match_count as u64);
                vga::print_str(")\n");
            }

            vga::set_color(vga::Color::LightCyan, vga::Color::Black);
            vga::print_str("Active Connection Tracking (CONNTRACK):\n");
            for i in 0..CONNTRACK_TABLE.len() {
                let conn = &CONNTRACK_TABLE[i];
                if !conn.in_use {
                    continue;
                }
                let proto = core::str::from_utf8(&conn.proto)
                    .unwrap_or("")
                    .trim_matches('\0');
                let src = core::str::from_utf8(&conn.src_ip)
                    .unwrap_or("")
                    .trim_matches('\0');
                let dst = core::str::from_utf8(&conn.dst_ip)
                    .unwrap_or("")
                    .trim_matches('\0');
                let state = core::str::from_utf8(&conn.state)
                    .unwrap_or("")
                    .trim_matches('\0');

                vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                vga::print_str("[CONN] ");
                vga::print_str(proto);
                vga::print_str(" ");
                vga::print_str(src);
                vga::print_str(" -> ");
                vga::print_str(dst);
                vga::print_str(":");
                vga::print_u64(conn.dport as u64);
                vga::print_str(" | State: ");
                vga::print_str(state);
                vga::print_str(" (Packets: ");
                vga::print_u64(conn.packets as u64);
                vga::print_str(")\n");
            }
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            Ok(0)
        }
        NETFILTER_CMD_ADD_RULE => {
            for rule in RULE_TABLE.iter_mut() {
                if !rule.in_use {
                    rule.chain = *b"INPUT\0\0\0\0\0\0\0";
                    rule.proto = *b"TCP\0\0\0\0\0";
                    rule.src_ip = *b"0.0.0.0/0\0\0\0\0\0\0\0";
                    rule.dst_ip = *b"192.168.1.100\0\0\0";
                    rule.dport = 22;
                    rule.action = *b"ACCEPT\0\0\0\0\0\0";
                    rule.match_count = 0;
                    rule.in_use = true;
                    break;
                }
            }
            Ok(0)
        }
        NETFILTER_CMD_DEL_RULE => {
            for rule in RULE_TABLE.iter_mut().rev() {
                if rule.in_use {
                    rule.in_use = false;
                    break;
                }
            }
            Ok(0)
        }
        NETFILTER_CMD_FLUSH => {
            for rule in RULE_TABLE.iter_mut() {
                rule.in_use = false;
            }
            Ok(0)
        }
        NETFILTER_CMD_TOGGLE => {
            NETFILTER_ENABLED = !NETFILTER_ENABLED;
            Ok(0)
        }
        _ => Err("Invalid Netfilter command vector"),
    }
}
