<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Anonymous Pipes & Zero-Copy Splice

This document specifies ring buffer pipes and zero-copy data streaming primitives in Keira Kernel.

---

## Technical Characteristics

* **Pipe Buffer Size**: 4096 bytes per pipe.
* **Synchronization**: Safe spinlock-guarded circular ring buffer.
* **Blocking Semantics**: Readers block when the pipe is empty; writers block when the pipe is full.

---

## Zero-Copy Splice API (`crates/ipc/src/pipe.rs`)

```rust
/// Create an anonymous unidirectional pipe returning (read_fd, write_fd).
pub fn sys_pipe() -> Result<(u32, u32), &'static str>;

/// Transfer up to len bytes from one file descriptor directly to another without userland roundtrips.
pub fn sys_splice(fd_in: u64, fd_out: u64, len: usize, flags: u32) -> Result<usize, &'static str>;

/// Splice userland memory slices directly into a kernel pipe.
pub fn sys_vmsplice(fd: u64, iov_ptr: u64, nr_segs: usize, flags: u32) -> Result<usize, &'static str>;
```
