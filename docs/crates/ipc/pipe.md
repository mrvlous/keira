<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Anonymous Pipes & Zero-Copy Splice

Documentation for pipes in [`crates/ipc/src/pipe/`](../../../crates/ipc/src/pipe).

## System Calls
- `sys_splice` (Syscall 36): Moves data between two file descriptors (such as a pipe and a network socket) without copying data to user space memory.
- `sys_vmsplice` (Syscall 37): Maps user pages directly into pipe buffers.
