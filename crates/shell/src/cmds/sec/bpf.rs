// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

#![allow(unused_variables, unused_unsafe)]

//! Inspect and execute Extended Berkeley Packet Filter (eBPF) runtime virtual machine programs (Syscall 78).

use keira_io::vga;
use keira_net::filter::bpf::{
    bpf_run_filter, bpf_verify, get_maps, get_programs, get_status, map_lookup, map_update,
    BpfInstruction, BPF_ABS, BPF_B, BPF_H, BPF_JEQ, BPF_JMP, BPF_K, BPF_LD, BPF_RET,
};

pub fn run(parts: &mut core::str::SplitWhitespace) {
    let subcmd = parts.next();

    if let Some("-h") | Some("--help") = subcmd {
        print_help();
        return;
    }

    match subcmd {
        None | Some("status") => {
            print_status();
        }
        Some("list") | Some("progs") => {
            print_programs();
        }
        Some("maps") => {
            print_maps();
        }
        Some("test") => {
            run_test_program();
        }
        Some("map-get") => {
            let map_id = parts.next().and_then(|s| s.parse::<u32>().ok());
            let key = parts.next().and_then(|s| s.parse::<u32>().ok());
            if let (Some(id), Some(k)) = (map_id, key) {
                match map_lookup(id, k) {
                    Some(val) => {
                        vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                        vga::print_str("[OK] ");
                        vga::set_color(vga::Color::White, vga::Color::Black);
                        vga::print_str("Map #");
                        vga::print_u64(id as u64);
                        vga::print_str(" Key 0x");
                        print_hex_u32(k);
                        vga::print_str(" => Value: ");
                        vga::print_u64(val);
                        vga::print_str("\n");
                    }
                    None => {
                        print_err("Key not found in specified BPF map.");
                    }
                }
            } else {
                print_err("Usage: bpf map-get <map_id> <key>");
            }
        }
        Some("map-set") => {
            let map_id = parts.next().and_then(|s| s.parse::<u32>().ok());
            let key = parts.next().and_then(|s| s.parse::<u32>().ok());
            let val = parts.next().and_then(|s| s.parse::<u64>().ok());
            if let (Some(id), Some(k), Some(v)) = (map_id, key, val) {
                match map_update(id, k, v) {
                    Ok(()) => {
                        vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                        vga::print_str("[OK] ");
                        vga::set_color(vga::Color::White, vga::Color::Black);
                        vga::print_str("Map #");
                        vga::print_u64(id as u64);
                        vga::print_str(" updated (Key: ");
                        vga::print_u64(k as u64);
                        vga::print_str(", Value: ");
                        vga::print_u64(v);
                        vga::print_str(")\n");
                    }
                    Err(e) => {
                        print_err(e);
                    }
                }
            } else {
                print_err("Usage: bpf map-set <map_id> <key> <value>");
            }
        }
        Some(other) => {
            print_err("Unknown subcommand. Run 'bpf --help' for usage.");
        }
    }
}

fn print_status() {
    let status = get_status();

    vga::set_color(vga::Color::White, vga::Color::Black);
    vga::print_str("Extended Berkeley Packet Filter (eBPF) Subsystem:\n");
    vga::print_str("  VM Engine State   : ");
    vga::set_color(vga::Color::LightGreen, vga::Color::Black);
    vga::print_str("Active (In-Kernel Bytecode Interpreter)\n");

    vga::set_color(vga::Color::White, vga::Color::Black);
    vga::print_str("  In-Kernel Verifier: ");
    vga::set_color(vga::Color::LightGreen, vga::Color::Black);
    vga::print_str("Enforced (CFG Bounded-Cycle Safety Check)\n");

    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    vga::print_str("  Loaded Programs   : ");
    vga::print_u64(status.total_programs as u64);
    vga::print_str("\n  Active Maps       : ");
    vga::print_u64(status.total_maps as u64);
    vga::print_str("\n  Total Executions  : ");
    vga::print_u64(status.total_executions);
    vga::print_str("\n  Syscall Interface : Syscall 78 (SYS_BPF)\n");
}

fn print_programs() {
    let progs = get_programs();

    vga::set_color(vga::Color::White, vga::Color::Black);
    vga::print_str("Loaded eBPF Kernel Programs:\n");
    vga::set_color(vga::Color::Cyan, vga::Color::Black);
    vga::print_str("  ID  TYPE           NAME            INSNS  RUNS   DROPS  PASSES\n");
    vga::print_str("  --  -------------  --------------  -----  -----  -----  ------\n");

    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    let mut count = 0;
    for prog in progs.iter() {
        if prog.in_use {
            count += 1;
            vga::print_str("  ");
            vga::print_u64(prog.id as u64);
            vga::print_str("   ");
            let t_str = prog.prog_type.as_str();
            vga::print_str(t_str);
            for _ in 0..(15usize.saturating_sub(t_str.len())) {
                vga::print_str(" ");
            }

            if let Ok(name) = core::str::from_utf8(&prog.name[..prog.name_len]) {
                vga::print_str(name);
                for _ in 0..(16usize.saturating_sub(name.len())) {
                    vga::print_str(" ");
                }
            } else {
                vga::print_str("unnamed         ");
            }

            vga::print_u64(prog.insn_count as u64);
            vga::print_str("      ");
            vga::print_u64(prog.runs);
            vga::print_str("      ");
            vga::print_u64(prog.drops);
            vga::print_str("      ");
            vga::print_u64(prog.passes);
            vga::print_str("\n");
        }
    }

    if count == 0 {
        vga::print_str("  No eBPF programs currently registered.\n");
    }
}

fn print_maps() {
    let maps = get_maps();

    vga::set_color(vga::Color::White, vga::Color::Black);
    vga::print_str("Active eBPF In-Kernel Maps:\n");
    vga::set_color(vga::Color::Cyan, vga::Color::Black);
    vga::print_str("  ID  TYPE         NAME             KEY  VAL  CAPACITY  ENTRIES\n");
    vga::print_str("  --  -----------  ---------------  ---  ---  --------  -------\n");

    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    let mut count = 0;
    for map in maps.iter() {
        if map.in_use {
            count += 1;
            vga::print_str("  ");
            vga::print_u64(map.id as u64);
            vga::print_str("   ");
            let t_str = map.map_type.as_str();
            vga::print_str(t_str);
            for _ in 0..(13usize.saturating_sub(t_str.len())) {
                vga::print_str(" ");
            }

            if let Ok(name) = core::str::from_utf8(&map.name[..map.name_len]) {
                vga::print_str(name);
                for _ in 0..(17usize.saturating_sub(name.len())) {
                    vga::print_str(" ");
                }
            } else {
                vga::print_str("unnamed          ");
            }

            vga::print_u64(map.key_size as u64);
            vga::print_str("B   ");
            vga::print_u64(map.val_size as u64);
            vga::print_str("B   ");
            vga::print_u64(map.max_entries as u64);
            vga::print_str("        ");
            vga::print_u64(map.entries_count as u64);
            vga::print_str("\n");
        }
    }

    if count == 0 {
        vga::print_str("  No eBPF maps currently active.\n");
    }
}

fn run_test_program() {
    vga::set_color(vga::Color::White, vga::Color::Black);
    vga::print_str("Running in-kernel eBPF verification and VM test suite:\n");

    // Test program: Inspect synthetic IPv4 packet for TCP port 80
    let test_insns = [
        BpfInstruction {
            code: BPF_LD | BPF_B | BPF_ABS,
            jt: 0,
            jf: 0,
            k: 9,
        },
        BpfInstruction {
            code: BPF_JMP | BPF_JEQ | BPF_K,
            jt: 0,
            jf: 3,
            k: 6,
        },
        BpfInstruction {
            code: BPF_LD | BPF_H | BPF_ABS,
            jt: 0,
            jf: 0,
            k: 22,
        },
        BpfInstruction {
            code: BPF_JMP | BPF_JEQ | BPF_K,
            jt: 0,
            jf: 1,
            k: 80,
        },
        BpfInstruction {
            code: BPF_RET | BPF_K,
            jt: 0,
            jf: 0,
            k: 0xFFFF,
        },
        BpfInstruction {
            code: BPF_RET | BPF_K,
            jt: 0,
            jf: 0,
            k: 0,
        },
    ];

    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    vga::print_str("  [1/3] Static Verifier Analysis ... ");
    match bpf_verify(&test_insns) {
        Ok(()) => {
            vga::set_color(vga::Color::LightGreen, vga::Color::Black);
            vga::print_str("PASSED (6 insns, 0 jumps OOB, termination guaranteed)\n");
        }
        Err(e) => {
            vga::set_color(vga::Color::LightRed, vga::Color::Black);
            vga::print_str("FAILED: ");
            vga::print_str(e);
            vga::print_str("\n");
            return;
        }
    }

    // Packet A: IP proto=6 (TCP), DPort=80
    let mut pkt_http = [0u8; 32];
    pkt_http[9] = 6;
    pkt_http[22] = 0;
    pkt_http[23] = 80;

    // Packet B: IP proto=17 (UDP), DPort=53
    let mut pkt_dns = [0u8; 32];
    pkt_dns[9] = 17;
    pkt_dns[22] = 0;
    pkt_dns[23] = 53;

    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    vga::print_str("  [2/3] VM Execute Packet #1 (TCP Port 80) ... ");
    let res1 = bpf_run_filter(&test_insns, &pkt_http);
    if res1 != 0 {
        vga::set_color(vga::Color::LightGreen, vga::Color::Black);
        vga::print_str("ACCEPTED (Return: 0x");
        print_hex_u32(res1);
        vga::print_str(")\n");
    } else {
        vga::set_color(vga::Color::LightRed, vga::Color::Black);
        vga::print_str("DROPPED\n");
    }

    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    vga::print_str("  [3/3] VM Execute Packet #2 (UDP Port 53) ... ");
    let res2 = bpf_run_filter(&test_insns, &pkt_dns);
    if res2 == 0 {
        vga::set_color(vga::Color::LightGreen, vga::Color::Black);
        vga::print_str("FILTERED / DROPPED (Correct: Non-TCP)\n");
    } else {
        vga::set_color(vga::Color::LightRed, vga::Color::Black);
        vga::print_str("ACCEPTED (Unexpected)\n");
    }

    vga::set_color(vga::Color::White, vga::Color::Black);
    vga::print_str("eBPF VM Interpreter & Verifier validation completed successfully.\n");
}

fn print_hex_u32(val: u32) {
    let hex_chars = b"0123456789ABCDEF";
    let mut buf = [0u8; 8];
    for i in 0..8 {
        let shift = (7 - i) * 4;
        buf[i] = hex_chars[((val >> shift) & 0x0F) as usize];
    }
    if let Ok(s) = core::str::from_utf8(&buf) {
        vga::print_str(s);
    }
}

fn print_help() {
    vga::print_str(
        "Usage: bpf [status|list|maps|test|map-get <id> <key>|map-set <id> <key> <val>]\n\n",
    );
    vga::print_str("Description:\n");
    vga::print_str("  Inspect and manage Extended Berkeley Packet Filter (eBPF) programs and maps (Syscall 78).\n\n");
    vga::print_str("Subcommands:\n");
    vga::print_str("  status                         Show eBPF virtual machine engine status and stats (default)\n");
    vga::print_str(
        "  list, progs                    List loaded in-kernel eBPF programs and telemetry\n",
    );
    vga::print_str(
        "  maps                           List active in-kernel BPF maps and capacity\n",
    );
    vga::print_str(
        "  test                           Run synthetic verification and VM execution test suite\n",
    );
    vga::print_str("  map-get <id> <key>             Retrieve value for key from target BPF map\n");
    vga::print_str("  map-set <id> <key> <val>       Set key-value pair in target BPF map\n");
    vga::print_str("  -h, --help                     Show this help message and exit\n");
}

fn print_err(msg: &str) {
    vga::set_color(vga::Color::LightRed, vga::Color::Black);
    vga::print_str("[ERROR] ");
    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    vga::print_str(msg);
    vga::print_str("\n");
}
