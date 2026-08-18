<!--
SPDX-License-Identifier: GPL-2.0-only

Keira Kernel - Operating System Kernel
Copyright (C) 2026 Moh. Ananda Firmansyah Putra
-->

# eBPF JIT Compiler Engine

This document details in-kernel eBPF bytecode compilation into native x86_64 machine code instructions in Keira Kernel.

---

## 1. Subsystem Overview

Keira Kernel implements an in-kernel eBPF JIT compiler ([bpf_jit.rs](../../kernel/src/net/bpf_jit.rs), **Syscall 59 `sys_bpf_jit`**) translating eBPF bytecode into native Ring 0 x86_64 machine instructions for zero-overhead packet filtering and event tracing.

---

## 2. System Call Interface

```c
// Syscall 59: Compile eBPF bytecode to native x86_64 machine code
long sys_bpf_jit(const void *insn_ptr, unsigned long insn_cnt);
```

---

## 3. Kernel APIs

*   `pub fn sys_bpf_jit(insn_ptr: *const u8, insn_cnt: usize) -> Result<u64, &'static str>`: Allocates executable JIT buffer and emits x86_64 opcodes.
