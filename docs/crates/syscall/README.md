<!-- SPDX-License-Identifier: GPL-2.0-only -->

# `keira-syscall` - System Call Dispatcher & Vector Table

The `keira-syscall` crate implements the complete 62-vector system call table, Ring 3 MSR configuration, TSS stack transitions, and CPU exception routing.

## Submodules

- [`table.md`](table.md): Complete 62 system call vector catalog.
- [`dispatcher.md`](dispatcher.md): Syscall entry, argument decoding, and return ABI.
- [`tss.md`](tss.md): Task State Segment (TSS) & Ring 3 IST stack.
- [`exception.md`](exception.md): CPU exception routing (`#PF`, `#GP`, `#DF`).
