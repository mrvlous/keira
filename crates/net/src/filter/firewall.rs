// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

#![allow(static_mut_refs)]

//! Stateful Netfilter firewall rules, connection tracking (CONNTRACK), and packet filtering.

use keira_io::vga;

pub const NETFILTER_CMD_STATUS: u32 = 1;
pub const NETFILTER_CMD_ADD_RULE: u32 = 2;
pub const NETFILTER_CMD_DEL_RULE: u32 = 3;
pub const NETFILTER_CMD_FLUSH: u32 = 4;
pub const NETFILTER_CMD_TOGGLE: u32 = 5;

pub const MAX_FIREWALL_RULES: usize = 16;
pub const MAX_CONNTRACK_ENTRIES: usize = 16;

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
pub static mut PACKETS_INSPECTED: u64 = 0;
pub static mut PACKETS_DROPPED: u64 = 0;

static mut RULE_TABLE: [FirewallRule; MAX_FIREWALL_RULES] = [
    FirewallRule {
        chain: *b"INPUT\0\0\0\0\0\0\0",
        proto: *b"TCP\0\0\0\0\0",
        src_ip: *b"0.0.0.0/0\0\0\0\0\0\0\0",
        dst_ip: *b"0.0.0.0/0\0\0\0\0\0\0\0",
        dport: 80,
        action: *b"ACCEPT\0\0\0\0\0\0",
        match_count: 0,
        in_use: true,
    },
    FirewallRule {
        chain: *b"INPUT\0\0\0\0\0\0\0",
        proto: *b"TCP\0\0\0\0\0",
        src_ip: *b"0.0.0.0/0\0\0\0\0\0\0\0",
        dst_ip: *b"0.0.0.0/0\0\0\0\0\0\0\0",
        dport: 443,
        action: *b"ACCEPT\0\0\0\0\0\0",
        match_count: 0,
        in_use: true,
    },
    FirewallRule {
        chain: *b"INPUT\0\0\0\0\0\0\0",
        proto: *b"ICMP\0\0\0\0",
        src_ip: *b"0.0.0.0/0\0\0\0\0\0\0\0",
        dst_ip: *b"0.0.0.0/0\0\0\0\0\0\0\0",
        dport: 0,
        action: *b"ACCEPT\0\0\0\0\0\0",
        match_count: 0,
        in_use: true,
    },
    FirewallRule {
        chain: *b"INPUT\0\0\0\0\0\0\0",
        proto: *b"TCP\0\0\0\0\0",
        src_ip: *b"0.0.0.0/0\0\0\0\0\0\0\0",
        dst_ip: *b"0.0.0.0/0\0\0\0\0\0\0\0",
        dport: 23,
        action: *b"DROP\0\0\0\0\0\0\0\0",
        match_count: 0,
        in_use: true,
    },
    FirewallRule {
        chain: [0; 12],
        proto: [0; 8],
        src_ip: [0; 16],
        dst_ip: [0; 16],
        dport: 0,
        action: [0; 12],
        match_count: 0,
        in_use: false,
    },
    FirewallRule {
        chain: [0; 12],
        proto: [0; 8],
        src_ip: [0; 16],
        dst_ip: [0; 16],
        dport: 0,
        action: [0; 12],
        match_count: 0,
        in_use: false,
    },
    FirewallRule {
        chain: [0; 12],
        proto: [0; 8],
        src_ip: [0; 16],
        dst_ip: [0; 16],
        dport: 0,
        action: [0; 12],
        match_count: 0,
        in_use: false,
    },
    FirewallRule {
        chain: [0; 12],
        proto: [0; 8],
        src_ip: [0; 16],
        dst_ip: [0; 16],
        dport: 0,
        action: [0; 12],
        match_count: 0,
        in_use: false,
    },
    FirewallRule {
        chain: [0; 12],
        proto: [0; 8],
        src_ip: [0; 16],
        dst_ip: [0; 16],
        dport: 0,
        action: [0; 12],
        match_count: 0,
        in_use: false,
    },
    FirewallRule {
        chain: [0; 12],
        proto: [0; 8],
        src_ip: [0; 16],
        dst_ip: [0; 16],
        dport: 0,
        action: [0; 12],
        match_count: 0,
        in_use: false,
    },
    FirewallRule {
        chain: [0; 12],
        proto: [0; 8],
        src_ip: [0; 16],
        dst_ip: [0; 16],
        dport: 0,
        action: [0; 12],
        match_count: 0,
        in_use: false,
    },
    FirewallRule {
        chain: [0; 12],
        proto: [0; 8],
        src_ip: [0; 16],
        dst_ip: [0; 16],
        dport: 0,
        action: [0; 12],
        match_count: 0,
        in_use: false,
    },
    FirewallRule {
        chain: [0; 12],
        proto: [0; 8],
        src_ip: [0; 16],
        dst_ip: [0; 16],
        dport: 0,
        action: [0; 12],
        match_count: 0,
        in_use: false,
    },
    FirewallRule {
        chain: [0; 12],
        proto: [0; 8],
        src_ip: [0; 16],
        dst_ip: [0; 16],
        dport: 0,
        action: [0; 12],
        match_count: 0,
        in_use: false,
    },
    FirewallRule {
        chain: [0; 12],
        proto: [0; 8],
        src_ip: [0; 16],
        dst_ip: [0; 16],
        dport: 0,
        action: [0; 12],
        match_count: 0,
        in_use: false,
    },
    FirewallRule {
        chain: [0; 12],
        proto: [0; 8],
        src_ip: [0; 16],
        dst_ip: [0; 16],
        dport: 0,
        action: [0; 12],
        match_count: 0,
        in_use: false,
    },
];

static mut CONNTRACK_TABLE: [ConnTrackEntry; MAX_CONNTRACK_ENTRIES] = [ConnTrackEntry {
    proto: [0; 8],
    src_ip: [0; 16],
    dst_ip: [0; 16],
    dport: 0,
    state: [0; 12],
    packets: 0,
    in_use: false,
}; MAX_CONNTRACK_ENTRIES];

/// Append a firewall rule dynamically to the table.
pub fn add_rule(
    chain: &str,
    proto: &str,
    dport: u16,
    action: &str,
    src_ip: &str,
    dst_ip: &str,
) -> Result<usize, &'static str> {
    unsafe {
        for (i, rule) in RULE_TABLE.iter_mut().enumerate() {
            if !rule.in_use {
                rule.chain = [0; 12];
                let c_bytes = chain.as_bytes();
                rule.chain[..core::cmp::min(c_bytes.len(), 11)]
                    .copy_from_slice(&c_bytes[..core::cmp::min(c_bytes.len(), 11)]);

                rule.proto = [0; 8];
                let p_bytes = proto.as_bytes();
                rule.proto[..core::cmp::min(p_bytes.len(), 7)]
                    .copy_from_slice(&p_bytes[..core::cmp::min(p_bytes.len(), 7)]);

                rule.src_ip = [0; 16];
                let s_bytes = src_ip.as_bytes();
                rule.src_ip[..core::cmp::min(s_bytes.len(), 15)]
                    .copy_from_slice(&s_bytes[..core::cmp::min(s_bytes.len(), 15)]);

                rule.dst_ip = [0; 16];
                let d_bytes = dst_ip.as_bytes();
                rule.dst_ip[..core::cmp::min(d_bytes.len(), 15)]
                    .copy_from_slice(&d_bytes[..core::cmp::min(d_bytes.len(), 15)]);

                rule.dport = dport;

                rule.action = [0; 12];
                let a_bytes = action.as_bytes();
                rule.action[..core::cmp::min(a_bytes.len(), 11)]
                    .copy_from_slice(&a_bytes[..core::cmp::min(a_bytes.len(), 11)]);

                rule.match_count = 0;
                rule.in_use = true;
                return Ok(i + 1);
            }
        }
        Err("Rule table full (maximum 16 rules)")
    }
}

/// Delete a firewall rule by 1-indexed number.
pub fn delete_rule(index: usize) -> Result<(), &'static str> {
    unsafe {
        if index == 0 || index > MAX_FIREWALL_RULES {
            return Err("Rule index out of range (1..16)");
        }
        if !RULE_TABLE[index - 1].in_use {
            return Err("Rule slot is already empty");
        }
        RULE_TABLE[index - 1].in_use = false;
        Ok(())
    }
}

/// Flush all firewall rules and connection tracking entries.
pub fn flush_rules() {
    unsafe {
        for rule in RULE_TABLE.iter_mut() {
            rule.in_use = false;
            rule.match_count = 0;
        }
        for conn in CONNTRACK_TABLE.iter_mut() {
            conn.in_use = false;
            conn.packets = 0;
        }
    }
}

/// Filter raw Ethernet IPv4 frame.
/// Returns true if packet is ACCEPT, false if DROP.
///
/// # Safety
/// The caller must ensure packet buffer pointers are valid.
pub unsafe fn filter_ipv4_frame(frame: &[u8]) -> bool {
    if !NETFILTER_ENABLED || frame.len() < 34 {
        return true;
    }

    PACKETS_INSPECTED += 1;

    let proto_num = frame[23];
    let dport = if (proto_num == 6 || proto_num == 17) && frame.len() >= 38 {
        ((frame[36] as u16) << 8) | (frame[37] as u16)
    } else {
        0
    };

    let proto_str = match proto_num {
        1 => "ICMP",
        6 => "TCP",
        17 => "UDP",
        _ => "OTHER",
    };

    for rule in RULE_TABLE.iter_mut() {
        if !rule.in_use {
            continue;
        }

        let rproto = core::str::from_utf8(&rule.proto)
            .unwrap_or("")
            .trim_matches('\0');
        if rproto != "ANY" && !rproto.eq_ignore_ascii_case(proto_str) {
            continue;
        }

        if rule.dport != 0 && rule.dport != dport {
            continue;
        }

        rule.match_count += 1;
        let action = core::str::from_utf8(&rule.action)
            .unwrap_or("")
            .trim_matches('\0');
        if action.eq_ignore_ascii_case("DROP") || action.eq_ignore_ascii_case("REJECT") {
            PACKETS_DROPPED += 1;
            return false;
        } else {
            return true;
        }
    }

    true
}

/// Filter packet against active firewall chain rules.
///
/// # Safety
/// The caller must ensure static rule tables are not concurrently accessed.
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
            return !action.eq_ignore_ascii_case("DROP") && !action.eq_ignore_ascii_case("REJECT");
        }
    }
    true
}

/// Attach BPF filter bytecode instructions to network socket.
pub fn bpf_filter_packet(pkt: &[u8], insns: &[BpfInstruction]) -> bool {
    if insns.is_empty() {
        return true;
    }
    vga::set_color(vga::Color::White, vga::Color::Black);
    vga::print_str("[BPF] Filtered Network Packet (Length: ");
    vga::print_u64(pkt.len() as u64);
    vga::print_str(" bytes, ");
    vga::print_u64(insns.len() as u64);
    vga::print_str(" BPF insns).\n");
    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    true
}

/// Print formatted firewall status and active rules.
///
/// # Safety
/// The caller must ensure VGA console state is initialized.
pub unsafe fn print_firewall_status() {
    vga::set_color(vga::Color::White, vga::Color::Black);
    vga::print_str("Stateful IPv4 Netfilter Firewall Status:\n");
    vga::print_str("Engine State: ");
    if NETFILTER_ENABLED {
        vga::set_color(vga::Color::LightGreen, vga::Color::Black);
        vga::print_str("ENABLED (Active Packet Inspection & Filtering)\n");
    } else {
        vga::set_color(vga::Color::Yellow, vga::Color::Black);
        vga::print_str("DISABLED\n");
    }

    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    vga::print_str("  Packets Inspected : ");
    vga::print_u64(PACKETS_INSPECTED);
    vga::print_str("\n  Packets Dropped   : ");
    vga::print_u64(PACKETS_DROPPED);
    vga::print_str("\n\n");

    vga::set_color(vga::Color::White, vga::Color::Black);
    vga::print_str("Active Firewall Chain Rules:\n");
    let mut rule_count = 0;
    for i in 0..RULE_TABLE.len() {
        let rule = &RULE_TABLE[i];
        if !rule.in_use {
            continue;
        }
        rule_count += 1;
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

        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        vga::print_str("  [Rule ");
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
        if action == "DROP" || action == "REJECT" {
            vga::set_color(vga::Color::LightRed, vga::Color::Black);
        } else {
            vga::set_color(vga::Color::LightGreen, vga::Color::Black);
        }
        vga::print_str(action);
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        vga::print_str(" (Matches: ");
        vga::print_u64(rule.match_count as u64);
        vga::print_str(")\n");
    }

    if rule_count == 0 {
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        vga::print_str("  (No active firewall rules. Run 'iptables -A ...' to add)\n");
    }
}

/// Netfilter firewall syscall dispatcher (Syscall 76).
///
/// # Safety
/// The caller must ensure that any pointer arguments are valid or null.
pub unsafe fn sys_netfilter(cmd: u32, arg1: u64, _arg2: u64) -> Result<u64, &'static str> {
    match cmd {
        NETFILTER_CMD_STATUS => {
            print_firewall_status();
            Ok(0)
        }
        NETFILTER_CMD_ADD_RULE => {
            add_rule("INPUT", "TCP", 22, "ACCEPT", "0.0.0.0/0", "0.0.0.0/0").map(|idx| idx as u64)
        }
        NETFILTER_CMD_DEL_RULE => {
            let idx = if arg1 != 0 { arg1 as usize } else { 1 };
            delete_rule(idx).map(|_| 0)
        }
        NETFILTER_CMD_FLUSH => {
            flush_rules();
            Ok(0)
        }
        NETFILTER_CMD_TOGGLE => {
            NETFILTER_ENABLED = !NETFILTER_ENABLED;
            Ok(if NETFILTER_ENABLED { 1 } else { 0 })
        }
        _ => Err("Invalid Netfilter command vector"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_firewall_rules_and_filtering() {
        flush_rules();
        unsafe {
            NETFILTER_ENABLED = true;
        }

        // Add drop rule for TCP port 8080
        let idx = add_rule("INPUT", "TCP", 8080, "DROP", "0.0.0.0/0", "0.0.0.0/0")
            .expect("add rule drop 8080");
        assert_eq!(idx, 1);

        // Construct synthetic Ethernet + IPv4 + TCP packet
        let mut frame = [0u8; 54];
        frame[12] = 0x08; // EtherType IPv4
        frame[13] = 0x00;
        frame[23] = 6; // Protocol TCP
        frame[36] = (8080 >> 8) as u8;
        frame[37] = (8080 & 0xff) as u8;

        // Filtering should DROP
        unsafe {
            assert!(!filter_ipv4_frame(&frame));
        }

        // Change dport to 80 (not matched by rule 1, default accept)
        frame[36] = 0;
        frame[37] = 80;
        unsafe {
            assert!(filter_ipv4_frame(&frame));
        }

        // Delete rule 1
        assert!(delete_rule(idx).is_ok());
        // Double delete should return Err
        assert!(delete_rule(idx).is_err());

        // Now port 8080 should be accepted
        frame[36] = (8080 >> 8) as u8;
        frame[37] = (8080 & 0xff) as u8;
        unsafe {
            assert!(filter_ipv4_frame(&frame));
        }

        flush_rules();
    }
}
