<!-- SPDX-License-Identifier: GPL-2.0-only -->

# CPU Exception Routing & Page Fault Handler

Documentation for exceptions in [`crates/syscall/src/exception.rs`](../../../crates/syscall/src/exception.rs).

## Handled Exceptions
- Vector `14` (`#PF` Page Fault): Reads faulting virtual address from `CR2`, handles Copy-On-Write (COW) or demands paging.
- Vector `13` (`#GP` General Protection Fault): Catches privilege level violations or non-canonical addresses.
- Vector `8` (`#DF` Double Fault): Emergency IST stack recovery.
