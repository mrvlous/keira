// SPDX-License-Identifier: GPL-2.0-only

//! Freestanding io_uring Completion Queue Entry (CQE).

/// Freestanding io_uring Completion Queue Entry (CQE).
#[repr(C)]
#[derive(Copy, Clone, Debug, Default)]
pub struct CompletionQueueEntry {
    pub user_data: u64,
    pub res: i32,
    pub flags: u32,
}
