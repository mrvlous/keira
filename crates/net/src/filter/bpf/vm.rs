// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Berkeley Packet Filter (BPF) kernel virtual machine and execution runtime.

#![allow(static_mut_refs)]

use crate::filter::bpf::insn::*;
use crate::filter::bpf::map::{create_map_internal, BpfMapType, BPF_MAPS};
use crate::filter::bpf::prog::{load_program_internal, BpfProgType, BPF_PROGRAMS};

/// Telemetry status snapshot of the in-kernel BPF virtual machine.
pub struct BpfStatus {
    pub engine_ready: bool,
    pub verifier_active: bool,
    pub total_programs: usize,
    pub total_maps: usize,
    pub total_executions: u64,
}

pub static mut BPF_INITIALIZED: bool = false;
pub static mut TOTAL_BPF_EXECUTIONS: u64 = 0;

/// Ensure the BPF engine subsystem is initialized.
pub fn ensure_initialized() {
    unsafe {
        if !BPF_INITIALIZED {
            init();
        }
    }
}

/// Initialize the in-kernel eBPF VM engine, default telemetry maps, and built-in programs.
pub fn init() {
    unsafe {
        if BPF_INITIALIZED {
            return;
        }

        // Map 0: Packet drop telemetry counter array
        create_map_internal(0, 1, BpfMapType::Array, "drop_counters", 4, 8, 16);
        // Map 1: Port blacklist hash table
        create_map_internal(1, 2, BpfMapType::Hash, "port_blacklist", 4, 8, 16);
        if let Some(map) = BPF_MAPS.get_mut(1) {
            map.entries[0] = (23, 1); // Telnet blocked
            map.entries[1] = (445, 1); // SMB blocked
            map.entries_count = 2;
        }

        // Preload Program 1: TCP Port 80 Web traffic filter
        let web_filter = [
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
        load_program_internal(0, 1, BpfProgType::SocketFilter, "http_filter", &web_filter);

        BPF_INITIALIZED = true;
    }
}

/// Execute BPF program against an input packet slice in the kernel virtual machine.
pub fn bpf_run_filter(insns: &[BpfInstruction], packet: &[u8]) -> u32 {
    if insns.is_empty() {
        return 0xFFFF; // Default pass
    }

    unsafe {
        TOTAL_BPF_EXECUTIONS += 1;
    }

    let mut reg_a: u32 = 0;
    let mut reg_x: u32 = 0;
    let mut mem = [0u32; 16];
    let mut pc: usize = 0;
    let max_cycles = 256;
    let mut cycles = 0;

    while pc < insns.len() && cycles < max_cycles {
        cycles += 1;
        let insn = insns[pc];
        let class = insn.code & 0x07;

        match class {
            BPF_LD => {
                let size = insn.code & 0x18;
                let mode = insn.code & 0xe0;

                if mode == BPF_IMM {
                    reg_a = insn.k;
                } else if mode == BPF_ABS {
                    let k = insn.k as usize;
                    reg_a = if size == BPF_B && k < packet.len() {
                        packet[k] as u32
                    } else if size == BPF_H && k + 1 < packet.len() {
                        ((packet[k] as u32) << 8) | (packet[k + 1] as u32)
                    } else if size == BPF_W && k + 3 < packet.len() {
                        ((packet[k] as u32) << 24)
                            | ((packet[k + 1] as u32) << 16)
                            | ((packet[k + 2] as u32) << 8)
                            | (packet[k + 3] as u32)
                    } else {
                        0
                    };
                } else if mode == BPF_MEM {
                    let k = (insn.k as usize) % 16;
                    reg_a = mem[k];
                } else if mode == BPF_LEN {
                    reg_a = packet.len() as u32;
                }
                pc += 1;
            }
            BPF_LDX => {
                let mode = insn.code & 0xe0;
                if mode == BPF_IMM {
                    reg_x = insn.k;
                } else if mode == BPF_MEM {
                    let k = (insn.k as usize) % 16;
                    reg_x = mem[k];
                } else if mode == BPF_LEN {
                    reg_x = packet.len() as u32;
                }
                pc += 1;
            }
            BPF_ST => {
                let k = (insn.k as usize) % 16;
                mem[k] = reg_a;
                pc += 1;
            }
            BPF_STX => {
                let k = (insn.k as usize) % 16;
                mem[k] = reg_x;
                pc += 1;
            }
            BPF_ALU => {
                let op = insn.code & 0xf0;
                let src = insn.code & 0x08;
                let v = if src == BPF_X { reg_x } else { insn.k };

                match op {
                    BPF_ADD => reg_a = reg_a.wrapping_add(v),
                    BPF_SUB => reg_a = reg_a.wrapping_sub(v),
                    BPF_MUL => reg_a = reg_a.wrapping_mul(v),
                    BPF_DIV => {
                        if v != 0 {
                            reg_a /= v;
                        }
                    }
                    BPF_OR => reg_a |= v,
                    BPF_AND => reg_a &= v,
                    BPF_LSH => reg_a = reg_a.wrapping_shl(v & 31),
                    BPF_RSH => reg_a = reg_a.wrapping_shr(v & 31),
                    _ => {}
                }
                pc += 1;
            }
            BPF_JMP => {
                let op = insn.code & 0xf0;
                let src = insn.code & 0x08;
                let v = if src == BPF_X { reg_x } else { insn.k };

                if op == BPF_JA {
                    pc += 1 + (insn.k as usize);
                } else {
                    let cond = match op {
                        BPF_JEQ => reg_a == v,
                        BPF_JGT => reg_a > v,
                        BPF_JGE => reg_a >= v,
                        BPF_JSET => (reg_a & v) != 0,
                        _ => false,
                    };
                    if cond {
                        pc += 1 + (insn.jt as usize);
                    } else {
                        pc += 1 + (insn.jf as usize);
                    }
                }
            }
            BPF_RET => {
                let src = insn.code & 0x18;
                if src == BPF_K {
                    return insn.k;
                } else {
                    return reg_a;
                }
            }
            _ => {
                pc += 1;
            }
        }
    }

    0 // Timeout or drop
}

/// Retrieve general eBPF engine status.
pub fn get_status() -> BpfStatus {
    ensure_initialized();
    unsafe {
        let progs_count = BPF_PROGRAMS.iter().filter(|p| p.in_use).count();
        let maps_count = BPF_MAPS.iter().filter(|m| m.in_use).count();
        BpfStatus {
            engine_ready: true,
            verifier_active: true,
            total_programs: progs_count,
            total_maps: maps_count,
            total_executions: TOTAL_BPF_EXECUTIONS,
        }
    }
}
