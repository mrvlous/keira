// SPDX-License-Identifier: GPL-2.0-only
/*
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra <mrvlous@proton.me>
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 */

//! Bounded Ring 0 workqueue implementation (`workqueue_struct`).

use core::sync::atomic::{AtomicU64, Ordering};

use keira_core::sync::{IrqSpinLock, LockRank};

use crate::workqueue::work::{
    DelayedWork, WorkStruct, WORK_STRUCT_ACTIVE, WORK_STRUCT_COMPLETED, WORK_STRUCT_DELAYED,
    WORK_STRUCT_PENDING,
};

/// Maximum concurrent deferred work descriptors per queue.
pub const MAX_WORK_ITEMS_PER_QUEUE: usize = 32;

static NEXT_WORK_ID: AtomicU64 = AtomicU64::new(1);

/// Snapshot of a pending deferred work descriptor for telemetry inspection.
#[derive(Copy, Clone, Debug)]
pub struct WorkItemSnapshot {
    /// Assigned work identifier.
    pub id: u64,
    /// Diagnostic label identifying subsystem ownership.
    pub name: &'static str,
    /// State bitmask (`WORK_STRUCT_*`).
    pub flags: u32,
    /// Earliest execution timestamp in monotonic milliseconds.
    pub execute_at_ms: u64,
}

impl WorkItemSnapshot {
    /// Constructs an empty work item snapshot.
    pub const fn empty() -> Self {
        Self {
            id: 0,
            name: "",
            flags: 0,
            execute_at_ms: 0,
        }
    }
}

/// Bounded kernel workqueue coordinating deferred execution in worker context.
pub struct WorkQueueStruct {
    /// Diagnostic identifier name (e.g. `system_wq`, `events_unbound`).
    pub name: &'static str,
    /// Interrupt-safe spinlock protecting work slot mutations.
    pub lock: IrqSpinLock,
    /// Fixed slot storage avoiding dynamic Ring 0 heap allocations.
    pub items: [Option<WorkStruct>; MAX_WORK_ITEMS_PER_QUEUE],
    /// Number of currently queued items.
    pub pending_count: usize,
    /// Cumulative number of queued items.
    pub total_queued: u64,
    /// Cumulative number of executed callbacks.
    pub total_executed: u64,
    /// Cumulative number of cancelled items.
    pub total_cancelled: u64,
    /// Monotonic timestamp in milliseconds of the last completed execution.
    pub last_exec_tick: u64,
}

impl WorkQueueStruct {
    /// Constructs a new empty named workqueue without heap allocation.
    pub const fn new(name: &'static str) -> Self {
        Self {
            name,
            lock: IrqSpinLock::with_rank(LockRank::Scheduler),
            items: [None; MAX_WORK_ITEMS_PER_QUEUE],
            pending_count: 0,
            total_queued: 0,
            total_executed: 0,
            total_cancelled: 0,
            last_exec_tick: 0,
        }
    }

    /// Enqueues an immediate work descriptor using the current monotonic clock tick.
    pub fn queue_work(&mut self, work: WorkStruct) -> Result<u64, &'static str> {
        let now_ms = keira_arch::timers::pit::uptime_ms();
        self.queue_work_at(work, now_ms, now_ms, false)
    }

    /// Enqueues a delayed work descriptor scheduled for execution after `delay_ms`.
    pub fn queue_delayed_work(&mut self, mut dwork: DelayedWork) -> Result<u64, &'static str> {
        let now_ms = keira_arch::timers::pit::uptime_ms();
        let target_ms = now_ms.saturating_add(dwork.delay_ms);
        dwork.timer_pending = dwork.delay_ms > 0;
        self.queue_work_at(dwork.work, now_ms, target_ms, dwork.delay_ms > 0)
    }

    /// Enqueues a work descriptor with explicit enqueue and execution timestamps.
    pub fn queue_work_at(
        &mut self,
        mut work: WorkStruct,
        now_ms: u64,
        execute_at_ms: u64,
        delayed: bool,
    ) -> Result<u64, &'static str> {
        let work_id = NEXT_WORK_ID.fetch_add(1, Ordering::Relaxed);
        work.id = work_id;
        work.queued_tick_ms = now_ms;
        work.execute_at_ms = execute_at_ms;
        work.flags = WORK_STRUCT_PENDING;
        if delayed {
            work.flags |= WORK_STRUCT_DELAYED;
        }

        let _guard = self.lock.lock();
        for slot in self.items.iter_mut() {
            if slot.is_none() {
                *slot = Some(work);
                self.pending_count += 1;
                self.total_queued += 1;
                return Ok(work_id);
            }
        }

        Err("Kernel workqueue capacity exhausted")
    }

    /// Cancels a pending work descriptor matching `work_id` prior to execution.
    pub fn cancel_work(&mut self, work_id: u64) -> bool {
        let _guard = self.lock.lock();
        for slot in self.items.iter_mut() {
            if let Some(ref item) = slot {
                if item.id == work_id {
                    *slot = None;
                    if self.pending_count > 0 {
                        self.pending_count -= 1;
                    }
                    self.total_cancelled += 1;
                    return true;
                }
            }
        }
        false
    }

    /// Processes and executes all work items whose scheduled deadline has expired.
    pub fn flush_due_at(&mut self, now_ms: u64) -> usize {
        let mut batch = [WorkStruct::empty(); MAX_WORK_ITEMS_PER_QUEUE];
        let mut batch_len = 0usize;

        {
            let _guard = self.lock.lock();
            for slot in self.items.iter_mut() {
                if let Some(item) = slot {
                    if item.execute_at_ms <= now_ms {
                        let mut extracted = *item;
                        extracted.flags =
                            (extracted.flags & !WORK_STRUCT_PENDING) | WORK_STRUCT_ACTIVE;
                        batch[batch_len] = extracted;
                        batch_len += 1;
                        *slot = None;
                        if self.pending_count > 0 {
                            self.pending_count -= 1;
                        }
                    }
                }
            }
        }

        if batch_len == 0 {
            return 0;
        }

        // Execute callbacks strictly outside lock to prevent recursive lock acquisition.
        for item in batch.iter_mut().take(batch_len) {
            (item.func)(item.data);
            item.flags = (item.flags & !WORK_STRUCT_ACTIVE) | WORK_STRUCT_COMPLETED;
        }

        {
            let _guard = self.lock.lock();
            self.total_executed += batch_len as u64;
            self.last_exec_tick = now_ms;
        }

        batch_len
    }

    /// Processes due work items against the current PIT/HPET uptime clock.
    #[inline]
    pub fn flush_due(&mut self) -> usize {
        let now_ms = keira_arch::timers::pit::uptime_ms();
        self.flush_due_at(now_ms)
    }

    /// Forces immediate execution of every queued work item regardless of delay.
    pub fn flush_all(&mut self) -> usize {
        self.flush_due_at(u64::MAX)
    }

    /// Returns the number of currently queued items.
    #[inline]
    pub fn pending(&self) -> usize {
        self.pending_count
    }

    /// Returns true if no work items are queued.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.pending_count == 0
    }
}
