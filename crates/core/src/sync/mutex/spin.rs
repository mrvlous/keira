// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Scoped spin-mutex wrapping protected kernel data with RAII lock guard semantics.
//!
//! Provides mutual exclusion and synchronized access to shared data structures
//! in freestanding contexts where thread sleeping and OS scheduler blocks are unavailable.

use super::super::spinlock::raw::SpinLock;
use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};

/// Mutual exclusion container protecting inner data with an atomic spinlock.
pub struct SpinMutex<T> {
    lock: SpinLock,
    data: UnsafeCell<T>,
}

/// # Safety
///
/// `SpinMutex<T>` is safe to synchronize across threads if `T` implements `Send`.
unsafe impl<T: Send> Sync for SpinMutex<T> {}

/// # Safety
///
/// `SpinMutex<T>` is safe to transfer across threads if `T` implements `Send`.
unsafe impl<T: Send> Send for SpinMutex<T> {}

impl<T> SpinMutex<T> {
    /// Constructs a new `SpinMutex` protecting the provided data.
    pub const fn new(data: T) -> Self {
        Self {
            lock: SpinLock::new(),
            data: UnsafeCell::new(data),
        }
    }

    /// Acquires the mutex lock and returns an RAII guard providing mutable access.
    pub fn lock(&self) -> SpinMutexGuard<'_, T> {
        self.lock.lock();
        SpinMutexGuard { mutex: self }
    }

    /// Attempts to acquire the mutex lock without blocking.
    ///
    /// Returns `Some(SpinMutexGuard)` if acquired, or `None` if contested.
    pub fn try_lock(&self) -> Option<SpinMutexGuard<'_, T>> {
        if self.lock.try_lock() {
            Some(SpinMutexGuard { mutex: self })
        } else {
            None
        }
    }
}

/// Scoped RAII guard for `SpinMutex` providing safe mutable dereferencing.
///
/// Automatically releases the underlying spinlock when dropped out of scope.
pub struct SpinMutexGuard<'a, T> {
    mutex: &'a SpinMutex<T>,
}

impl<'a, T> Deref for SpinMutexGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        // Safety: Guard ownership guarantees exclusive access to the inner data
        unsafe { &*self.mutex.data.get() }
    }
}

impl<'a, T> DerefMut for SpinMutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        // Safety: Guard ownership guarantees exclusive access to the inner data
        unsafe { &mut *self.mutex.data.get() }
    }
}

impl<'a, T> Drop for SpinMutexGuard<'a, T> {
    fn drop(&mut self) {
        self.mutex.lock.unlock();
    }
}
