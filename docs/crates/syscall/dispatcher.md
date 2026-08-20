<!-- SPDX-License-Identifier: GPL-2.0-only -->

# System Call Dispatcher & Entry ABI

Documentation for dispatcher in [`crates/syscall/src/dispatcher.rs`](../../../crates/syscall/src/dispatcher.rs).

## Register Passing Convention
- System Call Number: `RAX`
- Argument 1: `RDI`
- Argument 2: `RSI`
- Argument 3: `RDX`
- Argument 4: `R10`
- Argument 5: `R8`
- Argument 6: `R9`
- Return Value: `RAX` (negative value indicates error code)
