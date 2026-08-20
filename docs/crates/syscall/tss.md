<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Task State Segment (TSS) & Privilege Stacks

Documentation for TSS in [`crates/syscall/src/tss.rs`](../../../crates/syscall/src/tss.rs).

## Invariants
- `RSP0`: Dedicated 64-bit kernel stack pointer loaded on Ring 3 privilege transitions.
- `IST1`: Interrupt Stack Table vector used for Double Fault (`#DF`) exception recovery.
