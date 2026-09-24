// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Extended Berkeley Packet Filter (eBPF) program descriptors and registration.

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
