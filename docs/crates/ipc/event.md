<!-- SPDX-License-Identifier: GPL-2.0-only -->

# `epoll`, `eventfd` & `signalfd`

Documentation for event notification in [`crates/ipc/src/event/`](../../../crates/ipc/src/event).

## Features
- **`epoll` (`epoll.rs`)**: Scalable I/O event notification multiplexing (`sys_epoll_create`, `sys_epoll_ctl`).
- **`eventfd` (`eventfd.rs`)**: 64-bit integer event notification counter for thread synchronization (`sys_eventfd`).
- **`signalfd` (`eventfd.rs`)**: Delivers POSIX signals via file descriptor reads (`sys_signalfd`).
