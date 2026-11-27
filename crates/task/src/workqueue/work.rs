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

//! Deferred work descriptors (`work_struct` and `delayed_work`).

/// Callback function pointer executed in Ring 0 worker process context.
pub type WorkFn = fn(data: usize);

/// Work item is currently queued and awaiting worker execution.
pub const WORK_STRUCT_PENDING: u32 = 0x01;

/// Work item carries a delayed execution timestamp threshold.
pub const WORK_STRUCT_DELAYED: u32 = 0x02;

/// Work item is actively running inside a worker callback.
pub const WORK_STRUCT_ACTIVE: u32 = 0x04;

/// Work item completed execution cleanly.
pub const WORK_STRUCT_COMPLETED: u32 = 0x08;

fn noop_work_fn(_data: usize) {}

/// Deferred work item queued onto a Ring 0 kernel workqueue.
#[derive(Copy, Clone)]
pub struct WorkStruct {
    /// Monotonic queue-assigned work identifier.
    pub id: u64,
    /// Diagnostic label identifying subsystem ownership.
    pub name: &'static str,
    /// Deferred callback routine executed by worker daemon.
    pub func: WorkFn,
    /// Opaque payload forwarded to callback routine.
    pub data: usize,
    /// State bitmask (`WORK_STRUCT_*`).
    pub flags: u32,
    /// Enqueue timestamp in monotonic milliseconds.
    pub queued_tick_ms: u64,
    /// Earliest execution timestamp in monotonic milliseconds.
    pub execute_at_ms: u64,
}

impl WorkStruct {
    /// Constructs a new unbound immediate work descriptor.
    pub const fn new(name: &'static str, func: WorkFn, data: usize) -> Self {
        Self {
            id: 0,
            name,
            func,
            data,
            flags: 0,
            queued_tick_ms: 0,
            execute_at_ms: 0,
        }
    }

    /// Constructs an empty placeholder work descriptor.
    pub const fn empty() -> Self {
        Self::new("", noop_work_fn, 0)
    }

    /// Initializes a work descriptor in place (Linux `INIT_WORK` equivalent).
    pub fn init(&mut self, name: &'static str, func: WorkFn, data: usize) {
        *self = Self::new(name, func, data);
    }

    /// Returns true if the work item is currently queued.
    #[inline]
    pub const fn is_pending(&self) -> bool {
        (self.flags & WORK_STRUCT_PENDING) != 0
    }

    /// Returns true if the work item has a delayed schedule.
    #[inline]
    pub const fn is_delayed(&self) -> bool {
        (self.flags & WORK_STRUCT_DELAYED) != 0
    }
}

/// Delayed deferred work item scheduled after a millisecond interval.
#[derive(Copy, Clone)]
pub struct DelayedWork {
    /// Embedded base work descriptor.
    pub work: WorkStruct,
    /// Configured delay interval in milliseconds.
    pub delay_ms: u64,
    /// Indicates whether timer delay expiration is still pending.
    pub timer_pending: bool,
}

impl DelayedWork {
    /// Constructs a new delayed work descriptor (Linux `INIT_DELAYED_WORK` equivalent).
    pub const fn new(name: &'static str, func: WorkFn, data: usize, delay_ms: u64) -> Self {
        Self {
            work: WorkStruct {
                id: 0,
                name,
                func,
                data,
                flags: WORK_STRUCT_DELAYED,
                queued_tick_ms: 0,
                execute_at_ms: 0,
            },
            delay_ms,
            timer_pending: false,
        }
    }
}
