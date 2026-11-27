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

//! Host-side unit test suite for wait queues and condition synchronization.

use super::entry::WaitQueueEntry;
use super::queue::WaitQueueHead;
use super::registry::{format_waitqueue_stats, get_wait_queue_snapshots};
use crate::types::TaskState;

#[test]
fn test_wait_queue_entry_creation() {
    let entry = WaitQueueEntry::new(10, 0);
    assert_eq!(entry.task_id, 10);
    assert!(!entry.is_exclusive());
    assert!(!entry.is_woken());

    let excl_entry = WaitQueueEntry::exclusive(11);
    assert_eq!(excl_entry.task_id, 11);
    assert!(excl_entry.is_exclusive());
    assert!(!excl_entry.is_woken());
}

#[test]
fn test_wait_queue_head_lifecycle() {
    let mut wq = WaitQueueHead::new("test_wq");
    assert_eq!(wq.name, "test_wq");
    assert!(wq.is_empty());
    assert_eq!(wq.waiter_count(), 0);

    let entry = WaitQueueEntry::new(5, 0);
    wq.prepare_to_wait(entry, TaskState::Blocked);
    assert_eq!(wq.waiter_count(), 1);
    assert!(!wq.is_empty());

    // Duplicate prepare_to_wait should be idempotent
    wq.prepare_to_wait(entry, TaskState::Blocked);
    assert_eq!(wq.waiter_count(), 1);

    wq.finish_wait(5);
    assert_eq!(wq.waiter_count(), 0);
    assert!(wq.is_empty());
}

#[test]
fn test_wait_queue_wake_up_exclusive() {
    let mut wq = WaitQueueHead::new("test_excl");

    wq.prepare_to_wait(WaitQueueEntry::exclusive(1), TaskState::Blocked);
    wq.prepare_to_wait(WaitQueueEntry::exclusive(2), TaskState::Blocked);
    wq.prepare_to_wait(WaitQueueEntry::new(3, 0), TaskState::Blocked);
    assert_eq!(wq.waiter_count(), 3);

    // wake_up wakes all non-exclusive waiters plus at most 1 exclusive waiter
    let woken = wq.wake_up();
    assert_eq!(woken, 2); // Task 1 (exclusive) and Task 3 (non-exclusive)
    assert_eq!(wq.waiter_count(), 1); // Task 2 remains

    let woken2 = wq.wake_up();
    assert_eq!(woken2, 1); // Task 2 (exclusive)
    assert_eq!(wq.waiter_count(), 0);
}

#[test]
fn test_wait_queue_wake_up_all() {
    let mut wq = WaitQueueHead::new("test_all");

    for i in 1..=5 {
        wq.prepare_to_wait(WaitQueueEntry::exclusive(i), TaskState::Blocked);
    }
    assert_eq!(wq.waiter_count(), 5);

    let woken = wq.wake_up_all();
    assert_eq!(woken, 5);
    assert_eq!(wq.waiter_count(), 0);
    assert!(wq.is_empty());
}

#[test]
fn test_wait_queue_registry_and_format() {
    let snapshots = get_wait_queue_snapshots();
    assert_eq!(snapshots.len(), 6);
    assert_eq!(snapshots[0].name, "child_exit_wq");
    assert_eq!(snapshots[1].name, "pipe_read_wq");
    assert_eq!(snapshots[2].name, "pipe_write_wq");
    assert_eq!(snapshots[5].name, "workqueue_wq");

    let mut buf = [0u8; 1024];
    let len = format_waitqueue_stats(&mut buf);
    assert!(len > 0);

    let output = core::str::from_utf8(&buf[..len]).expect("Valid UTF-8 stats table");
    assert!(output.contains("# Name"));
    assert!(output.contains("child_exit_wq"));
    assert!(output.contains("pipe_read_wq"));
}
