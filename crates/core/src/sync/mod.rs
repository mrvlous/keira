// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Synchronization primitives for freestanding kernel environments.

pub mod irq;
pub mod irq_mutex;
pub mod irq_spinlock;
pub mod lock_order;
pub mod mutex;
pub mod spinlock;

pub use irq::{interrupts_enabled, irq_restore, irq_save, IrqState};
pub use irq_mutex::{IrqMutex, IrqMutexGuard};
pub use irq_spinlock::{IrqSpinLock, IrqSpinLockGuard};
pub use lock_order::{check_lock_order, record_lock_acquire, record_lock_release, LockRank};
pub use mutex::{SpinMutex, SpinMutexGuard};
pub use spinlock::SpinLock;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_irq_save_restore_lifecycle() {
        let state = irq_save();
        assert!(!interrupts_enabled());
        irq_restore(state);
        assert!(interrupts_enabled());
    }

    #[test]
    fn test_irq_spinlock_basic() {
        let lock = IrqSpinLock::new();
        assert!(!lock.is_locked());
        {
            let _guard = lock.lock();
            assert!(lock.is_locked());
            assert!(!interrupts_enabled());
        }
        assert!(!lock.is_locked());
        assert!(interrupts_enabled());
    }

    #[test]
    fn test_irq_mutex_deref_mut() {
        let mutex = IrqMutex::new(42);
        {
            let mut guard = mutex.lock();
            assert_eq!(*guard, 42);
            *guard = 100;
        }
        {
            let guard = mutex.lock();
            assert_eq!(*guard, 100);
        }
    }

    #[test]
    fn test_lock_hierarchy_ordering() {
        assert!(check_lock_order(LockRank::Pmm).is_ok());
        let prior = record_lock_acquire(LockRank::Pmm);
        // Rank 2 (Heap) after Rank 1 (Pmm) should be rejected
        assert!(check_lock_order(LockRank::Heap).is_err());
        record_lock_release(prior);
        assert!(check_lock_order(LockRank::Heap).is_ok());
    }
}
