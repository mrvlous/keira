// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Scoped spin-mutex wrapping inner data with RAII lock guard semantics.

use super::spinlock::SpinLock;
use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};

/// A mutual exclusion primitive safeguarding inner data with a spinlock.
pub struct SpinMutex<T> {
    lock: SpinLock,
    data: UnsafeCell<T>,
}

unsafe impl<T: Send> Sync for SpinMutex<T> {}
unsafe impl<T: Send> Send for SpinMutex<T> {}

impl<T> SpinMutex<T> {
    /// Create a new SpinMutex protecting the given data.
    pub const fn new(data: T) -> Self {
        Self {
            lock: SpinLock::new(),
            data: UnsafeCell::new(data),
        }
    }

    /// Lock the mutex and return an RAII guard providing mutable access.
    pub fn lock(&self) -> SpinMutexGuard<'_, T> {
        self.lock.lock();
        SpinMutexGuard { mutex: self }
    }

    /// Try to lock the mutex without spinning.
    pub fn try_lock(&self) -> Option<SpinMutexGuard<'_, T>> {
        if self.lock.try_lock() {
            Some(SpinMutexGuard { mutex: self })
        } else {
            None
        }
    }
}

/// An RAII implementation of a "scoped lock" of a SpinMutex.
pub struct SpinMutexGuard<'a, T> {
    mutex: &'a SpinMutex<T>,
}

impl<'a, T> Deref for SpinMutexGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.mutex.data.get() }
    }
}

impl<'a, T> DerefMut for SpinMutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.mutex.data.get() }
    }
}

impl<'a, T> Drop for SpinMutexGuard<'a, T> {
    fn drop(&mut self) {
        self.mutex.lock.unlock();
    }
}
