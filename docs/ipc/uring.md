<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Asynchronous I/O Ring Queues (`io_uring`)

This document details the lockless submission and completion ring buffer architecture implemented in Keira Kernel.

---

## Ring Buffer Architecture

```
Userland / Application                Kernel Async Worker
+----------------------+              +----------------------+
| Submission Queue (SQ)| === push ==> | Process SQE entries  |
+----------------------+              +----------------------+
                                                 ||
                                              complete
                                                 \/
+----------------------+              +----------------------+
| Completion Queue (CQ)| <== pop ===  | Post CQE results     |
+----------------------+              +----------------------+
```

---

## Core API (`crates/ipc/src/uring.rs`)

```rust
pub fn sys_io_uring_setup(entries: u32) -> Result<u32, &'static str>;
pub fn sys_io_uring_enter(ring_id: u32, to_submit: u32, min_complete: u32) -> Result<u32, &'static str>;
```
