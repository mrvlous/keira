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

//! Global wait queue registry and subsystem telemetry snapshots.

use core::fmt::Write;

use crate::wait::queue::{WaitQueueHead, MAX_WAITERS_PER_QUEUE};

/// Wait queue for child process exit notifications (parent waitpid).
pub static mut CHILD_EXIT_WQ: WaitQueueHead = WaitQueueHead::new("child_exit_wq");

/// Wait queue for anonymous pipe reader readiness.
pub static mut PIPE_READ_WQ: WaitQueueHead = WaitQueueHead::new("pipe_read_wq");

/// Wait queue for anonymous pipe writer capacity.
pub static mut PIPE_WRITE_WQ: WaitQueueHead = WaitQueueHead::new("pipe_write_wq");

/// Wait queue for background memory page reclamation daemon.
pub static mut KSWAPD_WQ: WaitQueueHead = WaitQueueHead::new("kswapd_wq");

/// Wait queue for background buffer writeback daemon.
pub static mut KFLUSHD_WQ: WaitQueueHead = WaitQueueHead::new("kflushd_wq");

/// Wait queue for background kernel workqueue executor daemon.
pub static mut WORKQUEUE_WQ: WaitQueueHead = WaitQueueHead::new("workqueue_wq");

/// Total number of system registered wait queues.
pub const REGISTERED_WAIT_QUEUES: usize = 6;

/// Telemetry snapshot of a single wait queue head.
#[derive(Copy, Clone, Debug)]
pub struct WaitQueueSnapshot {
    /// Diagnostic name identifier.
    pub name: &'static str,
    /// Number of tasks currently blocked on this queue.
    pub waiters_count: usize,
    /// Cumulative count of tasks that entered wait state.
    pub total_waits: u64,
    /// Cumulative count of tasks woken.
    pub total_wakeups: u64,
    /// Monotonic timestamp in milliseconds of last wakeup event.
    pub last_wake_tick: u64,
    /// Task identifiers currently waiting.
    pub waiting_pids: [usize; MAX_WAITERS_PER_QUEUE],
    /// Number of valid PIDs stored in waiting_pids.
    pub waiting_pids_len: usize,
}

impl WaitQueueSnapshot {
    /// Constructs an empty snapshot.
    pub const fn empty() -> Self {
        Self {
            name: "",
            waiters_count: 0,
            total_waits: 0,
            total_wakeups: 0,
            last_wake_tick: 0,
            waiting_pids: [0; MAX_WAITERS_PER_QUEUE],
            waiting_pids_len: 0,
        }
    }
}

/// Gathers snapshots of all registered kernel wait queues without dynamic allocation.
pub fn get_wait_queue_snapshots() -> [WaitQueueSnapshot; REGISTERED_WAIT_QUEUES] {
    let mut snapshots = [WaitQueueSnapshot::empty(); REGISTERED_WAIT_QUEUES];

    unsafe {
        let queues: [&mut WaitQueueHead; REGISTERED_WAIT_QUEUES] = [
            &mut *core::ptr::addr_of_mut!(CHILD_EXIT_WQ),
            &mut *core::ptr::addr_of_mut!(PIPE_READ_WQ),
            &mut *core::ptr::addr_of_mut!(PIPE_WRITE_WQ),
            &mut *core::ptr::addr_of_mut!(KSWAPD_WQ),
            &mut *core::ptr::addr_of_mut!(KFLUSHD_WQ),
            &mut *core::ptr::addr_of_mut!(WORKQUEUE_WQ),
        ];

        for (i, wq) in queues.into_iter().enumerate() {
            let _guard = wq.lock.lock();
            let mut pids = [0usize; MAX_WAITERS_PER_QUEUE];
            let mut pids_len = 0;

            for slot in wq.waiters.iter() {
                if let Some(ref entry) = slot {
                    if pids_len < MAX_WAITERS_PER_QUEUE {
                        pids[pids_len] = entry.task_id;
                        pids_len += 1;
                    }
                }
            }

            snapshots[i] = WaitQueueSnapshot {
                name: wq.name,
                waiters_count: wq.waiters_count,
                total_waits: wq.total_waits,
                total_wakeups: wq.total_wakeups,
                last_wake_tick: wq.last_wake_tick,
                waiting_pids: pids,
                waiting_pids_len: pids_len,
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

/// Formats wait queue telemetry table into destination buffer.
pub fn format_waitqueue_stats(buf: &mut [u8]) -> usize {
    let mut writer = SliceWriter { buf, offset: 0 };
    let _ = write!(
        writer,
        "{:<18} {:>8} {:>12} {:>12} {:>14}\n",
        "# Name", "Waiters", "Total Waits", "Wakeups", "Last Wake (ms)"
    );

    let snapshots = get_wait_queue_snapshots();
    for snap in &snapshots {
        let _ = write!(
            writer,
            "{:<18} {:>8} {:>12} {:>12} {:>14}\n",
            snap.name,
            snap.waiters_count,
            snap.total_waits,
            snap.total_wakeups,
            snap.last_wake_tick
        );
    }

    writer.offset
}
