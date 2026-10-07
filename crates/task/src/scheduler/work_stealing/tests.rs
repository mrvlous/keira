// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Unit tests for Chase-Lev work-stealing deque, per-CPU runqueues, and work balancing.

use super::*;

#[test]
fn test_deque_lifo_local_push_pop() {
    let deque = ChaseLevDeque::new();
    assert!(deque.is_empty());
    assert_eq!(deque.len(), 0);

    assert!(deque.push(10).is_ok());
    assert!(deque.push(20).is_ok());
    assert!(deque.push(30).is_ok());
    assert_eq!(deque.len(), 3);
    assert!(!deque.is_empty());

    // Owner pops from bottom in LIFO order
    assert_eq!(deque.pop(), Some(30));
    assert_eq!(deque.pop(), Some(20));
    assert_eq!(deque.pop(), Some(10));
    assert_eq!(deque.pop(), None);
    assert!(deque.is_empty());
    assert_eq!(deque.len(), 0);
}

#[test]
fn test_deque_fifo_steal() {
    let deque = ChaseLevDeque::new();
    assert!(deque.push(1).is_ok());
    assert!(deque.push(2).is_ok());
    assert!(deque.push(3).is_ok());

    // Remote thieves steal from top in FIFO order
    assert_eq!(deque.steal(), Some(1));
    assert_eq!(deque.steal(), Some(2));
    assert_eq!(deque.steal(), Some(3));
    assert_eq!(deque.steal(), None);
    assert!(deque.is_empty());
}

#[test]
fn test_deque_capacity_bounds() {
    let deque = ChaseLevDeque::new();
    for i in 0..DEQUE_CAPACITY {
        assert!(deque.push(i).is_ok());
    }
    assert_eq!(deque.len(), DEQUE_CAPACITY);

    // Exceeding capacity must return Err with rejected element
    assert_eq!(deque.push(999), Err(999));

    // Pop one item, then push must succeed
    assert_eq!(deque.pop(), Some(DEQUE_CAPACITY - 1));
    assert!(deque.push(100).is_ok());
}

#[test]
fn test_enqueue_deduplication() {
    let task_id = 42;
    // Task 0 must never be enqueued
    assert!(!enqueue_task(0, 0));

    // Valid task enqueuing
    assert!(enqueue_task(0, task_id));
    assert!(is_task_queued(task_id));

    // Duplicate enqueue attempts must be rejected
    assert!(!enqueue_task(0, task_id));
    assert!(!enqueue_task(1, task_id));

    // Pop releases the queued bit
    assert_eq!(pop_local(0), Some(task_id));
    assert!(!is_task_queued(task_id));

    // Can be enqueued again after being popped
    assert!(enqueue_task(1, task_id));
    assert_eq!(steal_from(1), Some(task_id));
    assert!(!is_task_queued(task_id));
}

#[test]
fn test_empty_pop_and_steal() {
    let deque = ChaseLevDeque::new();
    assert_eq!(deque.pop(), None);
    assert_eq!(deque.steal(), None);
    assert!(deque.is_empty());
}
