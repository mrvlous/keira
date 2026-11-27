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

//! Wait queue entry representations and condition flags.

/// Indicates the waiter requires exclusive wakeup (only one waiter woken).
pub const WQ_FLAG_EXCLUSIVE: u32 = 0x01;

/// Indicates the waiting task was explicitly woken by an event.
pub const WQ_FLAG_WOKEN: u32 = 0x02;

/// Bounded wait queue entry representing a sleeping task.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct WaitQueueEntry {
    /// Target task identifier (PID).
    pub task_id: usize,
    /// Waiter flags (exclusive, woken).
    pub flags: u32,
    /// Opaque private context payload.
    pub private: usize,
}

impl WaitQueueEntry {
    /// Constructs a new wait queue entry for the given task.
    pub const fn new(task_id: usize, flags: u32) -> Self {
        Self {
            task_id,
            flags,
            private: 0,
        }
    }

    /// Constructs an exclusive wait queue entry.
    pub const fn exclusive(task_id: usize) -> Self {
        Self::new(task_id, WQ_FLAG_EXCLUSIVE)
    }

    /// Returns true if this waiter is exclusive.
    #[inline]
    pub const fn is_exclusive(&self) -> bool {
        (self.flags & WQ_FLAG_EXCLUSIVE) != 0
    }

    /// Returns true if this waiter has been signaled.
    #[inline]
    pub const fn is_woken(&self) -> bool {
        (self.flags & WQ_FLAG_WOKEN) != 0
    }
}
