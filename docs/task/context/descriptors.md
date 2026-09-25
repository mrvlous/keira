<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Process Control Block (PCB) Layout

Each task is defined by a `Task` struct (`crates/task/src/types/task/`):
* Process ID (`pid`) and Parent PID (`ppid`).
* CPU Registers (`Context`).
* Memory Space: `CR3` page table address and VMA table.
* File Descriptors: Array of 32 open file/socket/pipe slots.
* Credentials: UID, GID, effective UID/GID.
