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

//! Host-side unit tests for Ring 0 kernel workqueues and deferred callbacks.

use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use super::queue::{WorkQueueStruct, MAX_WORK_ITEMS_PER_QUEUE};
use super::registry::{
    cancel_system_work, format_workqueue_stats, get_workqueue_snapshots, run_workqueue_self_test,
    schedule_delayed_work, schedule_work,
};
use super::work::{DelayedWork, WorkStruct};

static UNIT_COUNTER: AtomicUsize = AtomicUsize::new(0);
static TEST_MUTEX: AtomicBool = AtomicBool::new(false);

struct TestLock;

impl TestLock {
    fn acquire() -> Self {
        while TEST_MUTEX
            .compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            core::hint::spin_loop();
        }
        TestLock
    }
}

impl Drop for TestLock {
    fn drop(&mut self) {
        TEST_MUTEX.store(false, Ordering::Release);
    }
}

fn increment_worker(amount: usize) {
    UNIT_COUNTER.fetch_add(amount, Ordering::Relaxed);
}

#[test]
fn test_work_struct_initialization() {
    let mut work = WorkStruct::new("alpha_job", increment_worker, 7);
    assert_eq!(work.name, "alpha_job");
    assert_eq!(work.data, 7);
    assert!(!work.is_pending());
    assert!(!work.is_delayed());

    work.init("beta_job", increment_worker, 9);
    assert_eq!(work.name, "beta_job");
    assert_eq!(work.data, 9);
}

#[test]
fn test_workqueue_queue_flush_and_reentrant_safe() {
    let mut wq = WorkQueueStruct::new("unit_wq");
    assert!(wq.is_empty());

    UNIT_COUNTER.store(0, Ordering::Relaxed);

    let id1 = wq
        .queue_work(WorkStruct::new("job1", increment_worker, 3))
        .expect("queue job1");
    let id2 = wq
        .queue_work(WorkStruct::new("job2", increment_worker, 4))
        .expect("queue job2");

    assert_ne!(id1, id2);
    assert_eq!(wq.pending(), 2);
    assert_eq!(wq.total_queued, 2);

    let ran = wq.flush_due_at(100);
    assert_eq!(ran, 2);
    assert!(wq.is_empty());
    assert_eq!(wq.total_executed, 2);
    assert_eq!(UNIT_COUNTER.load(Ordering::Relaxed), 7);
}

#[test]
fn test_delayed_work_and_cancellation() {
    let mut wq = WorkQueueStruct::new("unit_delay_wq");
    UNIT_COUNTER.store(0, Ordering::Relaxed);

    let d1 = DelayedWork::new("future_job", increment_worker, 10, 50);
    let d1_id = wq
        .queue_work_at(d1.work, 100, 150, true)
        .expect("queue delayed");

    let d2 = DelayedWork::new("cancel_job", increment_worker, 20, 50);
    let d2_id = wq
        .queue_work_at(d2.work, 100, 150, true)
        .expect("queue cancel");

    assert!(wq.cancel_work(d2_id));
    assert!(!wq.cancel_work(d2_id));
    assert_eq!(wq.total_cancelled, 1);

    // At t=120 (< 150), delayed job must remain queued without executing
    assert_eq!(wq.flush_due_at(120), 0);
    assert_eq!(wq.pending(), 1);
    assert_eq!(UNIT_COUNTER.load(Ordering::Relaxed), 0);

    // At t=150, remaining delayed job executes
    assert_eq!(wq.flush_due_at(150), 1);
    assert_eq!(UNIT_COUNTER.load(Ordering::Relaxed), 10);
    assert!(!wq.cancel_work(d1_id));
}

#[test]
fn test_workqueue_capacity_bound() {
    let mut wq = WorkQueueStruct::new("unit_cap_wq");
    for _ in 0..MAX_WORK_ITEMS_PER_QUEUE {
        assert!(wq
            .queue_work(WorkStruct::new("fill", increment_worker, 1))
            .is_ok());
    }
    assert!(wq
        .queue_work(WorkStruct::new("overflow", increment_worker, 1))
        .is_err());
    assert_eq!(wq.flush_all(), MAX_WORK_ITEMS_PER_QUEUE);
}

#[test]
fn test_global_workqueue_registry_and_self_test() {
    let _guard = TestLock::acquire();

    let (before, after) = run_workqueue_self_test();
    assert_eq!(after, before + 1);

    let delayed_id = schedule_delayed_work("delayed_test", increment_worker, 5, 1000)
        .expect("schedule delayed work");
    assert!(cancel_system_work(delayed_id));

    let imm_id = schedule_work("cancel_imm", increment_worker, 2).expect("schedule imm");
    assert!(cancel_system_work(imm_id));

    let snapshots = get_workqueue_snapshots();
    assert_eq!(snapshots.len(), 2);
    assert_eq!(snapshots[0].name, "system_wq");
    assert_eq!(snapshots[1].name, "events_unbound");

    let mut buf = [0u8; 512];
    let len = format_workqueue_stats(&mut buf);
    let text = core::str::from_utf8(&buf[..len]).expect("utf8 stats");
    assert!(text.contains("# Workqueue"));
    assert!(text.contains("system_wq"));
    assert!(text.contains("events_unbound"));
}
