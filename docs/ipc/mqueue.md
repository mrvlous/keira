<!-- SPDX-License-Identifier: GPL-2.0-only -->

# POSIX Message Queues (`mqueue`)

This document specifies priority-ordered message passing queues in Keira Kernel.

---

## Message Queue Attributes

```rust
pub struct MqAttr {
    pub mq_flags: i64,
    pub mq_maxmsg: i64,    // Maximum number of messages (default: 10)
    pub mq_msgsize: i64,   // Maximum message size in bytes (default: 8192)
    pub mq_curmsgs: i64,   // Current messages waiting in queue
}
```

---

## Core API (`crates/ipc/src/mqueue.rs`)

```rust
pub fn sys_mq_open(name: *const u8, oflag: i32, mode: u32) -> Result<u64, u64>;
pub fn sys_mq_send(mqdes: i32, msg_ptr: *const u8, len: usize, prio: u32) -> Result<(), &'static str>;
pub fn sys_mq_receive(mqdes: i32, msg_ptr: *mut u8, len: usize, prio_out: *mut u32) -> Result<usize, &'static str>;
```
