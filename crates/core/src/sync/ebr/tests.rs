// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Unit test suite for Epoch-Based Reclamation (EBR) primitives.

use super::*;
use core::sync::atomic::{AtomicUsize, Ordering};

static RECLAIM_COUNTER: AtomicUsize = AtomicUsize::new(0);
static TEST_LOCK: crate::sync::SpinLock = crate::sync::SpinLock::new();

struct TestGuard;
impl Drop for TestGuard {
    fn drop(&mut self) {
        TEST_LOCK.unlock();
    }
}

fn lock_test() -> TestGuard {
    TEST_LOCK.lock();
    TestGuard
}

fn mock_reclaim(_val: usize) {
    RECLAIM_COUNTER.fetch_add(1, Ordering::SeqCst);
}

#[test]
fn test_ebr_pin_unpin_lifecycle() {
    let _lock = lock_test();
    let initial_epoch = current_epoch();
    {
        let guard = pin_participant(0);
        assert_eq!(guard.epoch(), initial_epoch);
        assert!(PARTICIPANTS[0].is_pinned());
    }
    assert!(!PARTICIPANTS[0].is_pinned());
}

#[test]
fn test_ebr_epoch_advancement_prevented_by_pinned_lagging_reader() {
    let _lock = lock_test();
    let start_epoch = current_epoch();
    let guard = pin_participant(1);
    assert_eq!(guard.epoch(), start_epoch);

    // If participant 1 is pinned at start_epoch, try_advance_epoch() can advance once to start_epoch + 1
    assert!(try_advance_epoch());
    assert_eq!(current_epoch(), start_epoch + 1);

    // Now participant 1 is pinned at start_epoch, which is < current_epoch (start_epoch + 1).
    // Advancement must now be prevented because participant 1 has not caught up!
    assert!(!try_advance_epoch());
    assert_eq!(current_epoch(), start_epoch + 1);

    // Drop the guard, unpinning participant 1
    drop(guard);
    assert!(!PARTICIPANTS[1].is_pinned());

    // Now advancement must succeed
    assert!(try_advance_epoch());
    assert_eq!(current_epoch(), start_epoch + 2);
}

#[test]
fn test_ebr_garbage_bag_lifecycle() {
    let _lock = lock_test();
    let mut bag = GarbageBag::new();
    RECLAIM_COUNTER.store(0, Ordering::SeqCst);

    let start_epoch = current_epoch();
    assert!(bag.retire(0x1000, Some(mock_reclaim)).is_ok());
    assert_eq!(bag.pending_count(), 1);

    // Immediate collection in same epoch should not reclaim yet (must be >= epoch + 2)
    assert_eq!(bag.collect(), 0);
    assert_eq!(RECLAIM_COUNTER.load(Ordering::SeqCst), 0);
    assert_eq!(bag.pending_count(), 1);

    // Advance 1 epoch
    assert!(try_advance_epoch());
    assert_eq!(current_epoch(), start_epoch + 1);
    assert_eq!(bag.collect(), 0);
    assert_eq!(RECLAIM_COUNTER.load(Ordering::SeqCst), 0);

    // Advance 2nd epoch
    assert!(try_advance_epoch());
    assert_eq!(current_epoch(), start_epoch + 2);

    // Now collection should reclaim the retired item
    assert_eq!(bag.collect(), 1);
    assert_eq!(RECLAIM_COUNTER.load(Ordering::SeqCst), 1);
    assert_eq!(bag.pending_count(), 0);
}
