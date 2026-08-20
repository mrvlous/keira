<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Stack Frame Pointer Unwinder

Documentation for callstack backtracing in [`crates/arch/src/debug/unwind.rs`](../../../crates/arch/src/debug/unwind.rs).

## Mechanism
Walks the linked list of stack frame pointers stored in RBP:
1. Captures current `RBP` and `RIP`.
2. Dereferences `*(RBP + 8)` to obtain the caller's return instruction pointer.
3. Dereferences `*RBP` to obtain the preceding frame pointer.
4. Validates that pointers reside within legitimate kernel stack virtual address boundaries (`0x100000` to `0x7FFFFFFFFFFF`) to prevent page faults during corrupt stack unwinds.
