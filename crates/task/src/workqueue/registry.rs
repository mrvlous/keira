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

//! Global system workqueue instances and telemetry snapshot providers.

use core::fmt::Write;
use core::sync::atomic::{AtomicU64, Ordering};

use crate::workqueue::queue::{WorkItemSnapshot, WorkQueueStruct, MAX_WORK_ITEMS_PER_QUEUE};
use crate::workqueue::work::{DelayedWork, WorkFn, WorkStruct};

/// Primary shared system workqueue for short deferred Ring 0 callbacks.
pub static mut SYSTEM_WQ: WorkQueueStruct = WorkQueueStruct::new("system_wq");

/// Secondary unbound workqueue for maintenance and delayed periodic jobs.
pub static mut SYSTEM_UNBOUND_WQ: WorkQueueStruct = WorkQueueStruct::new("events_unbound");

/// Total number of registered system workqueues.
pub const REGISTERED_WORKQUEUES: usize = 2;

static DIAGNOSTIC_PROBE_COUNT: AtomicU64 = AtomicU64::new(0);

/// Telemetry snapshot summarizing a registered workqueue and its pending items.
#[derive(Copy, Clone, Debug)]
pub struct WorkQueueSnapshot {
    /// Diagnostic name identifier.
    pub name: &'static str,
    /// Number of currently queued pending items.
    pub pending_count: usize,
    /// Cumulative count of queued work items.
    pub total_queued: u64,
    /// Cumulative count of executed callbacks.
    pub total_executed: u64,
    /// Cumulative count of cancelled items.
    pub total_cancelled: u64,
    /// Monotonic timestamp in milliseconds of the last completed execution.
    pub last_exec_tick: u64,
    /// Snapshot metadata for pending items.
    pub items: [WorkItemSnapshot; MAX_WORK_ITEMS_PER_QUEUE],
    /// Valid entries populated inside `items`.
    pub items_len: usize,
}

impl WorkQueueSnapshot {
    /// Constructs an empty workqueue snapshot.
    pub const fn empty() -> Self {
        Self {
            name: "",
            pending_count: 0,
            total_queued: 0,
            total_executed: 0,
            total_cancelled: 0,
            last_exec_tick: 0,
            items: [WorkItemSnapshot::empty(); MAX_WORK_ITEMS_PER_QUEUE],
            items_len: 0,
        }
    }
}

/// Schedules an immediate work item onto `SYSTEM_WQ`.
pub fn schedule_work(name: &'static str, func: WorkFn, data: usize) -> Result<u64, &'static str> {
    let work = WorkStruct::new(name, func, data);
    let work_id = unsafe {
        let wq = &mut *core::ptr::addr_of_mut!(SYSTEM_WQ);
        wq.queue_work(work)?
    };
    unsafe {
        let _ = (&mut *core::ptr::addr_of_mut!(crate::wait::WORKQUEUE_WQ)).wake_up();
    }
    Ok(work_id)
}

/// Schedules a delayed work item onto `SYSTEM_UNBOUND_WQ`.
pub fn schedule_delayed_work(
    name: &'static str,
    func: WorkFn,
    data: usize,
    delay_ms: u64,
) -> Result<u64, &'static str> {
    let dwork = DelayedWork::new(name, func, data, delay_ms);
    let work_id = unsafe {
        let wq = &mut *core::ptr::addr_of_mut!(SYSTEM_UNBOUND_WQ);
        wq.queue_delayed_work(dwork)?
    };
    if delay_ms == 0 {
        unsafe {
            let _ = (&mut *core::ptr::addr_of_mut!(crate::wait::WORKQUEUE_WQ)).wake_up();
        }
    }
    Ok(work_id)
}

/// Cancels a pending work item by identifier across registered system queues.
pub fn cancel_system_work(work_id: u64) -> bool {
    unsafe {
        let sys_wq = &mut *core::ptr::addr_of_mut!(SYSTEM_WQ);
        if sys_wq.cancel_work(work_id) {
            return true;
        }
        let unbound_wq = &mut *core::ptr::addr_of_mut!(SYSTEM_UNBOUND_WQ);
        unbound_wq.cancel_work(work_id)
    }
}

/// Processes all due work items across every registered system workqueue.
pub fn process_pending_workqueues() -> usize {
    unsafe {
        let sys_wq = &mut *core::ptr::addr_of_mut!(SYSTEM_WQ);
        let unbound_wq = &mut *core::ptr::addr_of_mut!(SYSTEM_UNBOUND_WQ);
        sys_wq.flush_due() + unbound_wq.flush_due()
    }
}

/// Flushes every queued item (including pending delayed items) across system queues.
pub fn flush_system_workqueues() -> usize {
    unsafe {
        let sys_wq = &mut *core::ptr::addr_of_mut!(SYSTEM_WQ);
        let unbound_wq = &mut *core::ptr::addr_of_mut!(SYSTEM_UNBOUND_WQ);
        sys_wq.flush_all() + unbound_wq.flush_all()
    }
}

fn diagnostic_probe_worker(delta: usize) {
    DIAGNOSTIC_PROBE_COUNT.fetch_add(delta.max(1) as u64, Ordering::Relaxed);
}

/// Runs a synchronous Ring 0 self-test by queuing and flushing a probe work item.
pub fn run_workqueue_self_test() -> (u64, u64) {
    let before = DIAGNOSTIC_PROBE_COUNT.load(Ordering::Relaxed);
    if schedule_work("kworker_probe", diagnostic_probe_worker, 1).is_ok() {
        let _ = process_pending_workqueues();
    }
    let after = DIAGNOSTIC_PROBE_COUNT.load(Ordering::Relaxed);
    (before, after)
}

/// Gathers non-allocating telemetry snapshots across all registered system workqueues.
pub fn get_workqueue_snapshots() -> [WorkQueueSnapshot; REGISTERED_WORKQUEUES] {
    let mut snapshots = [WorkQueueSnapshot::empty(); REGISTERED_WORKQUEUES];

    unsafe {
        let queues: [&mut WorkQueueStruct; REGISTERED_WORKQUEUES] = [
            &mut *core::ptr::addr_of_mut!(SYSTEM_WQ),
            &mut *core::ptr::addr_of_mut!(SYSTEM_UNBOUND_WQ),
        ];

        for (i, wq) in queues.into_iter().enumerate() {
            let _guard = wq.lock.lock();
            let mut item_snaps = [WorkItemSnapshot::empty(); MAX_WORK_ITEMS_PER_QUEUE];
            let mut items_len = 0usize;

            for slot in wq.items.iter() {
                if let Some(ref item) = slot {
                    if items_len < MAX_WORK_ITEMS_PER_QUEUE {
                        item_snaps[items_len] = WorkItemSnapshot {
                            id: item.id,
                            name: item.name,
                            flags: item.flags,
                            execute_at_ms: item.execute_at_ms,
                        };
                        items_len += 1;
                    }
                }
            }

            snapshots[i] = WorkQueueSnapshot {
                name: wq.name,
                pending_count: wq.pending_count,
                total_queued: wq.total_queued,
                total_executed: wq.total_executed,
                total_cancelled: wq.total_cancelled,
                last_exec_tick: wq.last_exec_tick,
                items: item_snaps,
                items_len,
            };
        }
    }

    snapshots
}

struct SliceWriter<'a> {
    buf: &'a mut [u8],
    offset: usize,
}

impl<'a> Write for SliceWriter<'a> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let bytes = s.as_bytes();
        let avail = self.buf.len().saturating_sub(self.offset);
        let to_write = bytes.len().min(avail);
        self.buf[self.offset..self.offset + to_write].copy_from_slice(&bytes[..to_write]);
        self.offset += to_write;
        Ok(())
    }
}

/// Formats workqueue telemetry table into destination buffer.
pub fn format_workqueue_stats(buf: &mut [u8]) -> usize {
    let mut writer = SliceWriter { buf, offset: 0 };
    let _ = write!(
        writer,
        "{:<16} {:>8} {:>10} {:>10} {:>10} {:>14}\n",
        "# Workqueue", "Pending", "Queued", "Executed", "Cancelled", "Last Run (ms)"
    );

    let snapshots = get_workqueue_snapshots();
    for snap in &snapshots {
        let _ = write!(
            writer,
            "{:<16} {:>8} {:>10} {:>10} {:>10} {:>14}\n",
            snap.name,
            snap.pending_count,
            snap.total_queued,
            snap.total_executed,
            snap.total_cancelled,
            snap.last_exec_tick
        );
    }

    writer.offset
}
