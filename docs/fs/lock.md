<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Advisory File Locks (`flock`)

This document specifies the advisory file locking mechanism preventing concurrent write corruption in Keira Kernel.

---

## Lock Types

| Lock Type | Constant | Semantics |
| :--- | :--- | :--- |
| `LOCK_SH` | Shared Lock | Multiple tasks may hold shared read locks concurrently |
| `LOCK_EX` | Exclusive Lock | Only one task may hold an exclusive write lock |
| `LOCK_UN` | Unlock | Releases held lock on target file descriptor |

---

## Core API (`crates/fs/src/lock/mod.rs`)

```rust
pub fn acquire_lock(fd: usize, is_exclusive: bool) -> Result<(), &'static str>;
pub fn release_lock(fd: usize);
```
