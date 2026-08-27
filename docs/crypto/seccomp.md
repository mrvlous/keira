<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Seccomp BPF System Call Filtering

This document specifies process sandboxing using Berkeley Packet Filter (BPF) programs in Keira Kernel.

---

## Filter Actions

* `SECCOMP_RET_ALLOW` (`0x7FFF0000`): Allow system call execution.
* `SECCOMP_RET_ERRNO` (`0x00050000`): Return specific POSIX errno directly without executing kernel handler.
* `SECCOMP_RET_KILL` (`0x00000000`): Terminate process immediately.

---

## Core API (`crates/task/src/security/mod.rs`)

```rust
pub fn sys_seccomp(op: u32, flags: u32, args_ptr: u64) -> Result<u64, u64>;
```
