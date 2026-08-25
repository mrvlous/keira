<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Task State Segment (TSS) & Privilege Stacks

Documentation for TSS in [`crates/syscall/src/tss.rs`](../../../crates/syscall/src/tss.rs).

## Invariants
- `RSP0`: Dedicated 64-bit kernel stack pointer loaded on Ring 3 privilege transitions (`x86_64`).
- `ESP0` & `SS0`: Dedicated 32-bit kernel stack pointer and kernel data segment (0x10) loaded on Ring 3 privilege transitions (`i686`).
- `IST1`: Interrupt Stack Table vector used for Double Fault (`#DF`) exception recovery in 64-bit mode.
