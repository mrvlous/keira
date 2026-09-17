// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Scoped interrupt-safe spin-mutex wrapping protected kernel data.

use super::irq_spinlock::{IrqSpinLock, IrqSpinLockGuard};
use super::lock_order::LockRank;
use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};

/// An interrupt-safe mutual exclusion primitive protecting inner data.
pub struct IrqMutex<T> {
    lock: IrqSpinLock,
    data: UnsafeCell<T>,
}

unsafe impl<T: Send> Sync for IrqMutex<T> {}
unsafe impl<T: Send> Send for IrqMutex<T> {}

impl<T> IrqMutex<T> {
    /// Create a new `IrqMutex` protecting the provided data with default rank.
    pub const fn new(data: T) -> Self {
        Self {
            lock: IrqSpinLock::new(),
            data: UnsafeCell::new(data),
        }
    }

    /// Create a new `IrqMutex` with an explicit hierarchy `LockRank`.
    pub const fn with_rank(data: T, rank: LockRank) -> Self {
        Self {
            lock: IrqSpinLock::with_rank(rank),
            data: UnsafeCell::new(data),
        }
    }

    /// Lock the mutex and return an RAII guard providing mutable access.
    pub fn lock(&self) -> IrqMutexGuard<'_, T> {
        let guard = self.lock.lock();
        IrqMutexGuard {
            _guard: guard,
            mutex: self,
        }
    }

    /// Try to lock the mutex without spinning.
    pub fn try_lock(&self) -> Option<IrqMutexGuard<'_, T>> {
        self.lock.try_lock().map(|guard| IrqMutexGuard {
            _guard: guard,
            mutex: self,
        })
    }
}

/// Scoped RAII guard for `IrqMutex` providing safe mutable dereferencing.
pub struct IrqMutexGuard<'a, T> {
    _guard: IrqSpinLockGuard<'a>,
    mutex: &'a IrqMutex<T>,
}

impl<'a, T> Deref for IrqMutexGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.mutex.data.get() }
    }
}

impl<'a, T> DerefMut for IrqMutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.mutex.data.get() }
    }
}
