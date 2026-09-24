// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Atomic test-and-set spinlock primitive for freestanding kernel synchronization.
//!
//! Implements a busy-waiting mutual exclusion primitive using atomic memory operations
//! with processor spin-loop pause hints to optimize cache line bouncing across SMP cores.

use core::sync::atomic::{AtomicBool, Ordering};

/// Atomic test-and-set mutual exclusion lock.
///
/// Suitable for short critical sections where sleeping or context switching is impermissible
/// (such as low-level scheduler queues, frame allocators, and hardware I/O dispatchers).
pub struct SpinLock {
    locked: AtomicBool,
}

impl Default for SpinLock {
    fn default() -> Self {
        Self::new()
    }
}

impl SpinLock {
    /// Constructs a new unlocked `SpinLock`.
    pub const fn new() -> Self {
        Self {
            locked: AtomicBool::new(false),
        }
    }

    /// Acquires the spinlock, busy-waiting until the lock becomes available.
    ///
    /// Utilizes `compare_exchange_weak` with acquire memory ordering on success
    /// and processor spin hints to minimize bus cache coherency traffic.
    #[inline]
    pub fn lock(&self) {
        while self
            .locked
            .compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            core::hint::spin_loop();
        }
    }

    /// Attempts to acquire the spinlock without blocking.
    ///
    /// Returns `true` if the lock was successfully acquired, `false` otherwise.
    #[inline]
    pub fn try_lock(&self) -> bool {
        self.locked
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_ok()
    }

    /// Releases the spinlock with release memory ordering.
    #[inline]
    pub fn unlock(&self) {
        self.locked.store(false, Ordering::Release);
    }

    /// Queries whether the spinlock is currently held by any execution thread.
    #[inline]
    pub fn is_locked(&self) -> bool {
        self.locked.load(Ordering::Relaxed)
    }
}
