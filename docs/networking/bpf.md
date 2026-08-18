<!--
SPDX-License-Identifier: GPL-2.0-only

Keira Kernel - Operating System Kernel
Copyright (C) 2026 Moh. Ananda Firmansyah Putra
-->

# Zero-Copy BPF (Berkeley Packet Filter) Engine

This document details the in-kernel BPF bytecode interpreter, raw packet filtering engine, and zero-copy network socket filtering in Keira Kernel.

---

## 1. Subsystem Overview

Keira Kernel provides an in-kernel BPF bytecode execution engine ([bpf.rs](../../kernel/src/net/bpf.rs)) allowing raw socket applications and network telemetry tools to attach filter programs directly to network interface drivers (Intel e1000 NIC).

---

## 2. BPF Instruction Set Architecture

The BPF interpreter executes 64-bit instruction structures (`BpfInstruction`):

```rust
pub struct BpfInstruction {
    pub code: u16,   // Instruction opcode (LD, ST, ALU, JMP, RET)
    pub jt: u8,      // Jump offset if true
    pub jf: u8,      // Jump offset if false
    pub k: u32,      // Generic operand value or packet offset
}
```

---

## 3. Kernel APIs

*   `pub fn filter_packet(pkt: &[u8], insns: &[BpfInstruction]) -> bool`: Evaluates BPF filter bytecode against incoming network packet headers, returning `true` if the packet matches filter criteria.
