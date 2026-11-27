// SPDX-License-Identifier: GPL-2.0-only
//
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

//! Linux-grade wait queues and sleep synchronization engine (`wait_queue_head_t`).

pub mod entry;
pub mod macros;
pub mod queue;
pub mod registry;

#[cfg(test)]
mod tests;

pub use entry::{WaitQueueEntry, WQ_FLAG_EXCLUSIVE, WQ_FLAG_WOKEN};
pub use queue::{WaitQueueHead, MAX_WAITERS_PER_QUEUE};
pub use registry::{
    format_waitqueue_stats, get_wait_queue_snapshots, WaitQueueSnapshot, CHILD_EXIT_WQ, KFLUSHD_WQ,
    KSWAPD_WQ, PIPE_READ_WQ, PIPE_WRITE_WQ, REGISTERED_WAIT_QUEUES, WORKQUEUE_WQ,
};
