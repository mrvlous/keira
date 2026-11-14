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

//! Mutual exclusion primitives (SpinMutex and IrqMutex).

pub mod spin {
    use crate::sync::spinlock::SpinLock;
    use core::cell::UnsafeCell;
    use core::ops::{Deref, DerefMut};

    pub struct SpinMutex<T> {
        lock: SpinLock,
        data: UnsafeCell<T>,
    }

    unsafe impl<T: Send> Sync for SpinMutex<T> {}

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
        /// Returns `Some(SpinMutexGuard)` if acquired or `None` if contested.
        pub fn try_lock(&self) -> Option<SpinMutexGuard<'_, T>> {
            if self.lock.try_lock() {
                Some(SpinMutexGuard { mutex: self })
            } else {
                None
            }
        }
    }

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
}

pub use self::spin::{SpinMutex, SpinMutexGuard};

pub mod irq_safe {
    use crate::sync::ordering::LockRank;
    use crate::sync::spinlock::{IrqSpinLock, IrqSpinLockGuard};
    use core::cell::UnsafeCell;
    use core::ops::{Deref, DerefMut};

    pub struct IrqMutex<T> {
        lock: IrqSpinLock,
        data: UnsafeCell<T>,
    }

    unsafe impl<T: Send> Sync for IrqMutex<T> {}

    unsafe impl<T: Send> Send for IrqMutex<T> {}

    impl<T> IrqMutex<T> {
        /// Constructs a new `IrqMutex` protecting the provided data with default rank.
        pub const fn new(data: T) -> Self {
            Self {
                lock: IrqSpinLock::new(),
                data: UnsafeCell::new(data),
            }
        }

        /// Constructs a new `IrqMutex` with an explicit hierarchy `LockRank`.
        pub const fn with_rank(data: T, rank: LockRank) -> Self {
            Self {
                lock: IrqSpinLock::with_rank(rank),
                data: UnsafeCell::new(data),
            }
        }

        /// Locks the mutex and returns an RAII guard providing mutable access.
        pub fn lock(&self) -> IrqMutexGuard<'_, T> {
            let guard = self.lock.lock();
            IrqMutexGuard {
                _guard: guard,
                mutex: self,
            }
        }

        /// Attempts to lock the mutex without spinning.
        pub fn try_lock(&self) -> Option<IrqMutexGuard<'_, T>> {
            self.lock.try_lock().map(|guard| IrqMutexGuard {
                _guard: guard,
                mutex: self,
            })
        }
    }

    pub struct IrqMutexGuard<'a, T> {
        _guard: IrqSpinLockGuard<'a>,
        mutex: &'a IrqMutex<T>,
    }

    impl<'a, T> Deref for IrqMutexGuard<'a, T> {
        type Target = T;

        fn deref(&self) -> &T {
            // Safety: Guard holds exclusive access guaranteed by IrqSpinLock
            unsafe { &*self.mutex.data.get() }
        }
    }

    impl<'a, T> DerefMut for IrqMutexGuard<'a, T> {
        fn deref_mut(&mut self) -> &mut T {
            // Safety: Guard holds exclusive access guaranteed by IrqSpinLock
            unsafe { &mut *self.mutex.data.get() }
        }
    }
}

pub use self::irq_safe::{IrqMutex, IrqMutexGuard};
