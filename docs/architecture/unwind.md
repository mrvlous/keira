<!--
SPDX-License-Identifier: GPL-2.0-only

Keira Kernel - Operating System Kernel
Copyright (C) 2026 Moh. Ananda Firmansyah Putra
-->

# Kernel Callstack Unwinder & Process Tracing Engine

This document details RBP/RSP pointer frame walking, kernel panic backtrace formatting, and process callstack tracing in Keira Kernel.

## 1. Unwinder Subsystem Architecture
The unwinder engine ([unwind.rs](../../kernel/src/arch/unwind.rs)) walks stack frame pointers to generate human-readable callstack backtraces during kernel panic debugging and process tracing.

*   **Stack Frame Pointer (RBP)**: Points to the base of the current stack frame.
*   **Return Address (RIP)**: Located at `RBP + 8` on the 64-bit x86_64 stack frame.
*   **Pointer Validation**: Ensures stack addresses fall within valid canonical memory bounds (`0x100000..0x7FFFFFFFFFFF`) with 8-byte alignment.

---

## 2. Unwind Algorithm (`unwind_from_frame`)
When an unhandled CPU exception (e.g. Page Fault Vector 14 or GPF Vector 13) occurs in kernel space:

```rust
pub unsafe fn unwind_from_frame(starting_rbp: u64, starting_rip: u64) {
    // Prints [#0] RIP: 0x... through [#N] RIP: 0x...
    // Output is routed simultaneously to VGA display and Serial COM1 port
}
```

---

## 3. Shell Commands
*   **`unwind`**: Triggers a manual kernel callstack unwind trace for the active shell execution context.
