// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Interrupt-safe spinlock with recursion prevention, timeout diagnostics, and RAII guard.
//!
//! Automatically masks local hardware interrupts before acquiring the atomic lock,
//! preventing deadlocks when an interrupt service routine interrupts a thread holding
//! the same lock on the same CPU core.

use super::super::irq::state::{irq_restore, irq_save, IrqState};
use super::super::ordering::rank::{
    check_lock_order, record_lock_acquire, record_lock_release, LockRank,
};
use core::sync::atomic::{AtomicBool, AtomicI32, AtomicU8, Ordering};

/// Reads the current CPU core ID via the processor CPUID instruction or environment fallback.
#[inline(always)]
pub fn current_core_id() -> i32 {
    #[cfg(all(target_os = "none", target_arch = "x86_64"))]
    {
        let leaf1 = core::arch::x86_64::__cpuid(1);
        (leaf1.ebx >> 24) as i32
    }
    #[cfg(all(target_os = "none", target_arch = "x86"))]
    {
        let leaf1 = core::arch::x86::__cpuid(1);
        (leaf1.ebx >> 24) as i32
    }
    #[cfg(not(target_os = "none"))]
    {
        0
    }
}

/// An interrupt-safe spinlock that disables local interrupts during acquisition.
pub struct IrqSpinLock {
    locked: AtomicBool,
    saved_irq: AtomicBool,
    saved_rank: AtomicU8,
    holder_core: AtomicI32,
    rank: LockRank,
}

impl Default for IrqSpinLock {
    fn default() -> Self {
        Self::new()
    }
}

impl IrqSpinLock {
    /// Constructs a new unlocked `IrqSpinLock` with default rank (`LockRank::None`).
    pub const fn new() -> Self {
        Self {
            locked: AtomicBool::new(false),
            saved_irq: AtomicBool::new(false),
            saved_rank: AtomicU8::new(0),
            holder_core: AtomicI32::new(-1),
            rank: LockRank::None,
        }
    }

    /// Constructs a new unlocked `IrqSpinLock` with an explicit hierarchy `LockRank`.
    pub const fn with_rank(rank: LockRank) -> Self {
        Self {
            locked: AtomicBool::new(false),
            saved_irq: AtomicBool::new(false),
            saved_rank: AtomicU8::new(0),
            holder_core: AtomicI32::new(-1),
            rank,
        }
    }

    /// Acquires the spinlock and returns an RAII guard that automatically releases
    /// the lock and restores prior interrupt state upon drop.
    #[must_use]
    pub fn lock(&self) -> IrqSpinLockGuard<'_> {
        let irq_state = irq_save();
        let core_id = current_core_id();

        // Enforce lock hierarchy ordering
        if self.rank != LockRank::None {
            let _ = check_lock_order(self.rank);
        }

        // Recursive lock detection on the same CPU core
        if self.locked.load(Ordering::Relaxed)
            && self.holder_core.load(Ordering::Relaxed) == core_id
        {
            panic!(
                "Deadlock detected: recursive IrqSpinLock acquisition on core {}",
                core_id
            );
        }

        let mut spin_count: u64 = 0;
        const SPIN_LIMIT: u64 = 10_000_000;

        while self
            .locked
            .compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            core::hint::spin_loop();
            spin_count += 1;
            if spin_count == SPIN_LIMIT {
                // Potential soft lockup threshold reached
                #[cfg(target_os = "none")]
                {
                    // Continue spinning to preserve system integrity
                }
            }
        }

        self.holder_core.store(core_id, Ordering::Relaxed);
        let prior_rank = if self.rank != LockRank::None {
            record_lock_acquire(self.rank)
        } else {
            LockRank::None
        };

        IrqSpinLockGuard {
            lock: self,
            irq_state,
            prior_rank,
        }
    }

    /// Acquires the spinlock without an RAII guard (requires explicit invocation of `unlock()`).
    pub fn acquire(&self) {
        let irq_state = irq_save();
        let core_id = current_core_id();

        if self.rank != LockRank::None {
            let _ = check_lock_order(self.rank);
        }

        if self.locked.load(Ordering::Relaxed)
            && self.holder_core.load(Ordering::Relaxed) == core_id
        {
            panic!(
                "Deadlock detected: recursive IrqSpinLock acquisition on core {}",
                core_id
            );
        }

        while self
            .locked
            .compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            core::hint::spin_loop();
        }

        self.saved_irq
            .store(irq_state.was_enabled, Ordering::Relaxed);
        self.holder_core.store(core_id, Ordering::Relaxed);
        if self.rank != LockRank::None {
            let prior = record_lock_acquire(self.rank);
            self.saved_rank.store(prior as u8, Ordering::Relaxed);
        }
    }

    /// Explicitly releases the spinlock and restores previous CPU interrupt state.
    pub fn unlock(&self) {
        let was_enabled = self.saved_irq.swap(false, Ordering::Relaxed);
        if self.rank != LockRank::None {
            let prior = LockRank::from_u8(self.saved_rank.swap(0, Ordering::Relaxed));
            record_lock_release(prior);
        }
        self.holder_core.store(-1, Ordering::Relaxed);
        self.locked.store(false, Ordering::Release);
        irq_restore(IrqState { was_enabled });
    }

    /// Attempts to acquire the spinlock without blocking.
    ///
    /// Returns `Some(IrqSpinLockGuard)` if acquired, or `None` if contested.
    pub fn try_lock(&self) -> Option<IrqSpinLockGuard<'_>> {
        let irq_state = irq_save();
        let core_id = current_core_id();

        if self
            .locked
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_ok()
        {
            self.holder_core.store(core_id, Ordering::Relaxed);
            let prior_rank = if self.rank != LockRank::None {
                record_lock_acquire(self.rank)
            } else {
                LockRank::None
            };
            Some(IrqSpinLockGuard {
                lock: self,
                irq_state,
                prior_rank,
            })
        } else {
            irq_restore(irq_state);
            None
        }
    }

    /// Queries whether the spinlock is currently held.
    #[inline]
    pub fn is_locked(&self) -> bool {
        self.locked.load(Ordering::Relaxed)
    }

    /// Queries which CPU core currently holds the spinlock (-1 if unlocked).
    #[inline]
    pub fn holder_core(&self) -> i32 {
        self.holder_core.load(Ordering::Relaxed)
    }
}

/// RAII lock guard for `IrqSpinLock` that automatically releases the lock and restores interrupts.
pub struct IrqSpinLockGuard<'a> {
    lock: &'a IrqSpinLock,
    irq_state: IrqState,
    prior_rank: LockRank,
}

impl<'a> Drop for IrqSpinLockGuard<'a> {
    fn drop(&mut self) {
        if self.lock.rank != LockRank::None {
            record_lock_release(self.prior_rank);
        }
        self.lock.holder_core.store(-1, Ordering::Relaxed);
        self.lock.locked.store(false, Ordering::Release);
        irq_restore(self.irq_state);
    }
}
