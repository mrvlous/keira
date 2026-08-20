<!-- SPDX-License-Identifier: GPL-2.0-only -->

# MAC Policies & Seccomp BPF Filter

Documentation for task security in [`crates/task/src/security.rs`](../../../crates/task/src/security.rs).

## Features
- **Mandatory Access Control (MAC)**: Enforces path-based access policies restricting unprivileged userland processes from writing to critical directories (`/config`, `/system`).
- **Seccomp (`sys_seccomp` - Syscall 42)**: Evaluates strict or BPF filter rules on every syscall entry, terminating unauthorized execution.
