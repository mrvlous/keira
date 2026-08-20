<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Virtual Memory Swap Pager

Documentation for swapping in [`crates/mem/src/swap/`](../../../crates/mem/src/swap).

## System Calls
- `sys_swapon(path_ptr: *const u8, swapflags: i32)` (Syscall 53): Activates a dedicated disk partition as swap space.
- `sys_swapoff(path_ptr: *const u8)` (Syscall 54): Deactivates swap partition and reloads dirty swapped pages into physical RAM.
