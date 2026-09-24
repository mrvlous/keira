// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Berkeley Packet Filter (BPF) static bytecode verifier.

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
