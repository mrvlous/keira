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

## Core API (`crates/ipc/src/mqueue/queue.rs`)

```rust
// Queue Management
pub unsafe fn mq_open(name: &str, flags: u32, max_msg: usize, msg_size: usize) -> Result<u32, &'static str>;
pub unsafe fn mq_send(name_or_id: &str, payload: &[u8], prio: u32) -> Result<(), &'static str>;
pub unsafe fn mq_receive(name_or_id: &str, out_buf: &mut [u8]) -> Result<(usize, u32), &'static str>;
pub unsafe fn mq_unlink(name: &str) -> Result<(), &'static str>;
pub unsafe fn mq_unlink_by_id(id: u32) -> Result<(), &'static str>;
pub unsafe fn get_mqueue_table() -> &'static [PosixMessageQueue];
pub unsafe fn get_mqueue_stats() -> (usize, usize);

// System Call Vector 58
pub unsafe fn sys_mq_open(name_ptr: *const u8, oflag: i32, mode: u32) -> Result<u64, &'static str>;
```

---

## Shell Integration

Inspect and interact with POSIX message queues via native shell commands:
- `mqueue status`: Display message queue subsystem statistics and capacity.
- `mqueue list`: Tabulate all active in-kernel message queues and backlog counts.
- `mqueue create <name>`: Create a new POSIX message queue (e.g. `/keira_mq0`).
- `mqueue send <queue> <msg>`: Enqueue message with default priority.
- `mqueue recv <queue>`: Dequeue highest priority message.
- `mqueue unlink <queue>`: Destroy and unlink message queue.
- `ipcs -q`: Display active queues in tabular format.
- `ipcrm -q <id|name>`: Remove queue by numeric ID or name.
