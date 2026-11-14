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

//! Subsystem module: bpf.

pub mod insn {
    // Instruction Classes
    pub const BPF_LD: u16 = 0x00;
    pub const BPF_LDX: u16 = 0x01;
    pub const BPF_ST: u16 = 0x02;
    pub const BPF_STX: u16 = 0x03;
    pub const BPF_ALU: u16 = 0x04;
    pub const BPF_JMP: u16 = 0x05;
    pub const BPF_RET: u16 = 0x06;
    pub const BPF_MISC: u16 = 0x07;

    // Size Modifiers
    pub const BPF_W: u16 = 0x00;
    pub const BPF_H: u16 = 0x08;
    pub const BPF_B: u16 = 0x10;

    // Mode Modifiers
    pub const BPF_IMM: u16 = 0x00;
    pub const BPF_ABS: u16 = 0x20;
    pub const BPF_IND: u16 = 0x40;
    pub const BPF_MEM: u16 = 0x60;
    pub const BPF_LEN: u16 = 0x80;

    // Source Operands
    pub const BPF_K: u16 = 0x00;
    pub const BPF_X: u16 = 0x08;
    pub const BPF_A: u16 = 0x10;

    // ALU Operations
    pub const BPF_ADD: u16 = 0x00;
    pub const BPF_SUB: u16 = 0x10;
    pub const BPF_MUL: u16 = 0x20;
    pub const BPF_DIV: u16 = 0x30;
    pub const BPF_OR: u16 = 0x40;
    pub const BPF_AND: u16 = 0x50;
    pub const BPF_LSH: u16 = 0x60;
    pub const BPF_RSH: u16 = 0x70;

    // Jump Operations
    pub const BPF_JA: u16 = 0x00;
    pub const BPF_JEQ: u16 = 0x10;
    pub const BPF_JGT: u16 = 0x20;
    pub const BPF_JGE: u16 = 0x30;
    pub const BPF_JSET: u16 = 0x40;

    pub const MAX_BPF_INSTRUCTIONS: usize = 32;

    /// Berkeley Packet Filter instruction format (Classic BPF / cBPF binary compatible).
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    pub struct BpfInstruction {
        pub code: u16,
        pub jt: u8,
        pub jf: u8,
        pub k: u32,
    }
}

pub mod map {
    #![allow(static_mut_refs)]

    pub const MAX_BPF_MAPS: usize = 8;
    pub const MAX_MAP_ENTRIES: usize = 16;

    /// In-kernel map types for stateful BPF programs.
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    pub enum BpfMapType {
        Hash = 1,
        Array = 2,
        ProgArray = 3,
    }

    impl BpfMapType {
        /// Return human-readable name of BPF map type.
        pub fn as_str(&self) -> &'static str {
            match self {
                BpfMapType::Hash => "hash",
                BpfMapType::Array => "array",
                BpfMapType::ProgArray => "prog_array",
            }
        }
    }

    /// Generic BPF map descriptor and static storage table.
    #[derive(Debug, Copy, Clone)]
    pub struct BpfMap {
        pub id: u32,
        pub map_type: BpfMapType,
        pub key_size: u32,
        pub val_size: u32,
        pub max_entries: u32,
        pub entries: [(u32, u64); MAX_MAP_ENTRIES],
        pub entries_count: usize,
        pub name: [u8; 16],
        pub name_len: usize,
        pub in_use: bool,
    }

    pub static mut BPF_MAPS: [BpfMap; MAX_BPF_MAPS] = [BpfMap {
        id: 0,
        map_type: BpfMapType::Hash,
        key_size: 4,
        val_size: 8,
        max_entries: MAX_MAP_ENTRIES as u32,
        entries: [(0, 0); MAX_MAP_ENTRIES],
        entries_count: 0,
        name: [0u8; 16],
        name_len: 0,
        in_use: false,
    }; MAX_BPF_MAPS];

    /// Helper to create an in-kernel map.
    ///
    /// # Safety
    /// Caller must ensure exclusive access to BPF_MAPS.
    pub unsafe fn create_map_internal(
        slot: usize,
        id: u32,
        map_type: BpfMapType,
        name: &str,
        key_size: u32,
        val_size: u32,
        max_entries: u32,
    ) {
        if slot >= MAX_BPF_MAPS {
            return;
        }
        let mut name_buf = [0u8; 16];
        let b = name.as_bytes();
        let to_copy = b.len().min(16);
        name_buf[..to_copy].copy_from_slice(&b[..to_copy]);

        BPF_MAPS[slot] = BpfMap {
            id,
            map_type,
            key_size,
            val_size,
            max_entries,
            entries: [(0, 0); MAX_MAP_ENTRIES],
            entries_count: 0,
            name: name_buf,
            name_len: to_copy,
            in_use: true,
        };
    }

    /// Get all registered BPF maps.
    pub fn get_maps() -> [BpfMap; MAX_BPF_MAPS] {
        crate::filter::bpf::vm::ensure_initialized();
        unsafe { BPF_MAPS }
    }

    /// Look up value in map.
    pub fn map_lookup(map_id: u32, key: u32) -> Option<u64> {
        crate::filter::bpf::vm::ensure_initialized();
        unsafe {
            for map in BPF_MAPS.iter() {
                if map.in_use && map.id == map_id {
                    for i in 0..map.entries_count {
                        if map.entries[i].0 == key {
                            return Some(map.entries[i].1);
                        }
                    }
                }
            }
        }
        None
    }

    /// Update key-value in map.
    pub fn map_update(map_id: u32, key: u32, val: u64) -> Result<(), &'static str> {
        crate::filter::bpf::vm::ensure_initialized();
        unsafe {
            for map in BPF_MAPS.iter_mut() {
                if map.in_use && map.id == map_id {
                    for i in 0..map.entries_count {
                        if map.entries[i].0 == key {
                            map.entries[i].1 = val;
                            return Ok(());
                        }
                    }
                    if map.entries_count < MAX_MAP_ENTRIES {
                        let idx = map.entries_count;
                        map.entries[idx] = (key, val);
                        map.entries_count += 1;
                        return Ok(());
                    } else {
                        return Err("Map is full");
                    }
                }
            }
        }
        Err("Map not found")
    }

    /// Delete key from map.
    pub fn map_delete(map_id: u32, key: u32) -> Result<(), &'static str> {
        crate::filter::bpf::vm::ensure_initialized();
        unsafe {
            for map in BPF_MAPS.iter_mut() {
                if map.in_use && map.id == map_id {
                    for i in 0..map.entries_count {
                        if map.entries[i].0 == key {
                            for j in i..(map.entries_count - 1) {
                                map.entries[j] = map.entries[j + 1];
                            }
                            map.entries_count -= 1;
                            return Ok(());
                        }
                    }
                    return Err("Key not found in map");
                }
            }
        }
        Err("Map not found")
    }
}

pub mod prog {
    #![allow(static_mut_refs)]

    use crate::filter::bpf::insn::{BpfInstruction, MAX_BPF_INSTRUCTIONS};

    pub const MAX_BPF_PROGS: usize = 8;

    /// BPF program hook attachment types.
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    pub enum BpfProgType {
        SocketFilter = 1,
        Kprobe = 2,
        Tracepoint = 3,
        Xdp = 4,
    }

    impl BpfProgType {
        /// Return human-readable name of BPF program type.
        pub fn as_str(&self) -> &'static str {
            match self {
                BpfProgType::SocketFilter => "socket_filter",
                BpfProgType::Kprobe => "kprobe",
                BpfProgType::Tracepoint => "tracepoint",
                BpfProgType::Xdp => "xdp",
            }
        }
    }

    /// In-kernel BPF program container and telemetry stats.
    #[derive(Debug, Copy, Clone)]
    pub struct BpfProgram {
        pub id: u32,
        pub prog_type: BpfProgType,
        pub name: [u8; 16],
        pub name_len: usize,
        pub insns: [BpfInstruction; MAX_BPF_INSTRUCTIONS],
        pub insn_count: usize,
        pub runs: u64,
        pub drops: u64,
        pub passes: u64,
        pub in_use: bool,
    }

    pub static mut BPF_PROGRAMS: [BpfProgram; MAX_BPF_PROGS] = [BpfProgram {
        id: 0,
        prog_type: BpfProgType::SocketFilter,
        name: [0u8; 16],
        name_len: 0,
        insns: [BpfInstruction {
            code: 0,
            jt: 0,
            jf: 0,
            k: 0,
        }; MAX_BPF_INSTRUCTIONS],
        insn_count: 0,
        runs: 0,
        drops: 0,
        passes: 0,
        in_use: false,
    }; MAX_BPF_PROGS];

    /// Helper to register a program.
    ///
    /// # Safety
    /// Caller must ensure exclusive access to BPF_PROGRAMS.
    pub unsafe fn load_program_internal(
        slot: usize,
        id: u32,
        prog_type: BpfProgType,
        name: &str,
        insns: &[BpfInstruction],
    ) {
        if slot >= MAX_BPF_PROGS || insns.len() > MAX_BPF_INSTRUCTIONS {
            return;
        }
        let mut name_buf = [0u8; 16];
        let b = name.as_bytes();
        let to_copy = b.len().min(16);
        name_buf[..to_copy].copy_from_slice(&b[..to_copy]);

        let mut insn_array = [BpfInstruction {
            code: 0,
            jt: 0,
            jf: 0,
            k: 0,
        }; MAX_BPF_INSTRUCTIONS];
        insn_array[..insns.len()].copy_from_slice(insns);

        BPF_PROGRAMS[slot] = BpfProgram {
            id,
            prog_type,
            name: name_buf,
            name_len: to_copy,
            insns: insn_array,
            insn_count: insns.len(),
            runs: 0,
            drops: 0,
            passes: 0,
            in_use: true,
        };
    }

    /// Get all registered BPF programs.
    pub fn get_programs() -> [BpfProgram; MAX_BPF_PROGS] {
        crate::filter::bpf::vm::ensure_initialized();
        unsafe { BPF_PROGRAMS }
    }
}

pub mod verifier {
    use crate::filter::bpf::insn::{
        BpfInstruction, BPF_ALU, BPF_DIV, BPF_JA, BPF_JMP, BPF_K, BPF_RET, MAX_BPF_INSTRUCTIONS,
    };

    /// Verify BPF program for safety: bounded instructions, valid jumps, no divide by zero.
    pub fn bpf_verify(insns: &[BpfInstruction]) -> Result<(), &'static str> {
        if insns.is_empty() {
            return Err("Empty BPF instruction set");
        }
        if insns.len() > MAX_BPF_INSTRUCTIONS {
            return Err("Instruction count exceeds maximum limit (32)");
        }

        let len = insns.len();
        let mut has_ret = false;

        for (idx, insn) in insns.iter().enumerate() {
            let class = insn.code & 0x07;

            if class == BPF_RET {
                has_ret = true;
            }

            if class == BPF_ALU {
                let op = insn.code & 0xf0;
                let src = insn.code & 0x08;
                if op == BPF_DIV && src == BPF_K && insn.k == 0 {
                    return Err("Verifier Error: Division by zero immediate");
                }
            }

            if class == BPF_JMP {
                let op = insn.code & 0xf0;
                if op == BPF_JA {
                    let target = idx + 1 + (insn.k as usize);
                    if target >= len {
                        return Err("Verifier Error: Unconditional jump target out of bounds");
                    }
                } else {
                    let target_t = idx + 1 + (insn.jt as usize);
                    let target_f = idx + 1 + (insn.jf as usize);
                    if target_t >= len || target_f >= len {
                        return Err("Verifier Error: Conditional jump target out of bounds");
                    }
                }
            }
        }

        if !has_ret {
            return Err("Verifier Error: Program missing terminating RET instruction");
        }

        Ok(())
    }
}

pub mod vm {
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

    /// Initialize the in-kernel eBPF VM engine, default telemetry maps and built-in programs.
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
                            if let Some(res) = reg_a.checked_div(v) {
                                reg_a = res;
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
}

pub use self::insn::*;
pub use self::map::*;
pub use self::prog::*;
pub use self::verifier::*;
pub use self::vm::*;
