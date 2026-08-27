<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Event Notification Descriptors (`eventfd`)

This document specifies 64-bit event counter notification descriptors used for inter-thread and inter-process signaling.

---

## Technical Specifications

* **Counter Type**: 64-bit unsigned integer (`u64`).
* **Flags Supported**:
  * `EFD_NONBLOCK`: Non-blocking read/write operations.
  * `EFD_SEMAPHORE`: Semaphore-style decrement on read.

---

## Core API (`crates/ipc/src/event.rs`)

```rust
pub fn sys_eventfd(initval: u32, flags: u32) -> Result<u64, u64>;
pub fn sys_epoll_create(size: i32) -> Result<u64, u64>;
pub fn sys_epoll_ctl(epfd: i32, op: i32, fd: i32, event_ptr: u64) -> Result<u64, u64>;
```
