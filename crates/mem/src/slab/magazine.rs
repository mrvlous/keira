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

//! Per-CPU magazines and global depot exchange for slab caches.

/// Number of object pointers stored in a single magazine.
pub const MAGAZINE_CAPACITY: usize = 32;

/// Fixed-size container of pre-allocated kernel object pointers.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Magazine {
    pub objects: [*mut u8; MAGAZINE_CAPACITY],
    pub rounds: usize,
}

unsafe impl Send for Magazine {}
unsafe impl Sync for Magazine {}

impl Default for Magazine {
    fn default() -> Self {
        Self::new()
    }
}

impl Magazine {
    /// Constructs a new empty magazine.
    pub const fn new() -> Self {
        Self {
            objects: [core::ptr::null_mut(); MAGAZINE_CAPACITY],
            rounds: 0,
        }
    }

    /// Pushes an object pointer onto the top of the magazine.
    #[inline]
    pub fn push(&mut self, ptr: *mut u8) -> Result<(), *mut u8> {
        if self.rounds < MAGAZINE_CAPACITY {
            self.objects[self.rounds] = ptr;
            self.rounds += 1;
            Ok(())
        } else {
            Err(ptr)
        }
    }

    /// Pops an object pointer from the top of the magazine in LIFO order.
    #[inline]
    pub fn pop(&mut self) -> Option<*mut u8> {
        if self.rounds > 0 {
            self.rounds -= 1;
            Some(self.objects[self.rounds])
        } else {
            None
        }
    }

    /// Returns  if the magazine contains no objects.
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.rounds == 0
    }

    /// Returns  if the magazine is at full capacity.
    #[inline]
    pub const fn is_full(&self) -> bool {
        self.rounds == MAGAZINE_CAPACITY
    }

    /// Returns the number of object pointers currently held.
    #[inline]
    pub const fn rounds(&self) -> usize {
        self.rounds
    }

    /// Clears the magazine pointers without freeing the underlying memory.
    #[inline]
    pub fn clear(&mut self) {
        self.rounds = 0;
    }
}

pub mod depot {
    use super::Magazine;
    use keira_core::sync::{IrqSpinLock, LockRank};

    pub const DEPOT_CAPACITY: usize = 64;

    pub struct GlobalDepot {
        full_stack: [Option<Magazine>; DEPOT_CAPACITY],
        full_count: usize,
        empty_stack: [Option<Magazine>; DEPOT_CAPACITY],
        empty_count: usize,
        lock: IrqSpinLock,
    }

    unsafe impl Send for GlobalDepot {}
    unsafe impl Sync for GlobalDepot {}

    impl Default for GlobalDepot {
        fn default() -> Self {
            Self::new()
        }
    }

    impl GlobalDepot {
        /// Constructs a new empty global depot.
        pub const fn new() -> Self {
            Self {
                full_stack: [const { None }; DEPOT_CAPACITY],
                full_count: 0,
                empty_stack: [const { None }; DEPOT_CAPACITY],
                empty_count: 0,
                lock: IrqSpinLock::with_rank(LockRank::Heap),
            }
        }

        /// Exchanges an empty magazine from a CPU core for a full magazine.
        ///
        /// If a full magazine is available, stores the empty magazine and returns `Some(full)`.
        /// Otherwise, stores the empty magazine (if space permits) and returns `None`.
        pub fn exchange_empty(&self, mut empty_mag: Magazine) -> Option<Magazine> {
            let _guard = self.lock.lock();
            let _ebr = keira_core::sync::ebr::pin();
            let ptr = self as *const Self as *mut Self;

            unsafe {
                empty_mag.clear();

                if (*ptr).full_count > 0 {
                    (*ptr).full_count -= 1;
                    let full = (*ptr).full_stack[(*ptr).full_count].take();

                    if (*ptr).empty_count < DEPOT_CAPACITY {
                        (*ptr).empty_stack[(*ptr).empty_count] = Some(empty_mag);
                        (*ptr).empty_count += 1;
                    }

                    full
                } else {
                    if (*ptr).empty_count < DEPOT_CAPACITY {
                        (*ptr).empty_stack[(*ptr).empty_count] = Some(empty_mag);
                        (*ptr).empty_count += 1;
                    }
                    None
                }
            }
        }

        /// Exchanges a full magazine from a CPU core for an empty magazine.
        ///
        /// If an empty magazine is available, stores the full magazine and returns `Some(empty)`.
        /// Otherwise, stores the full magazine (if space permits) and returns `Some(Magazine::new())`.
        pub fn exchange_full(&self, full_mag: Magazine) -> Option<Magazine> {
            let _guard = self.lock.lock();
            let _ebr = keira_core::sync::ebr::pin();
            let ptr = self as *const Self as *mut Self;

            unsafe {
                if (*ptr).full_count < DEPOT_CAPACITY {
                    (*ptr).full_stack[(*ptr).full_count] = Some(full_mag);
                    (*ptr).full_count += 1;
                }

                if (*ptr).empty_count > 0 {
                    (*ptr).empty_count -= 1;
                    (*ptr).empty_stack[(*ptr).empty_count].take()
                } else {
                    Some(Magazine::new())
                }
            }
        }

        /// Retrieves the count of full magazines currently buffered in the depot.
        pub fn full_count(&self) -> usize {
            let _guard = self.lock.lock();
            self.full_count
        }

        /// Retrieves the count of empty magazines currently buffered in the depot.
        pub fn empty_count(&self) -> usize {
            let _guard = self.lock.lock();
            self.empty_count
        }

        /// Drains all full magazines from the depot for reclamation or reaping.
        pub fn drain_full(&self) -> ([Option<Magazine>; DEPOT_CAPACITY], usize) {
            let _guard = self.lock.lock();
            let ptr = self as *const Self as *mut Self;
            unsafe {
                let count = (*ptr).full_count;
                let mut result = [const { None }; DEPOT_CAPACITY];
                for (i, slot) in result.iter_mut().enumerate().take(count) {
                    *slot = (*ptr).full_stack[i].take();
                }
                (*ptr).full_count = 0;
                (result, count)
            }
        }
    }
}

pub mod percpu {
    use super::{GlobalDepot, Magazine};
    use core::sync::atomic::{AtomicUsize, Ordering};

    pub const MAX_CPU_CORES: usize = 16;

    pub struct CpuDepot {
        active: Magazine,
        backup: Magazine,
        alloc_hits: AtomicUsize,
        free_hits: AtomicUsize,
        exchanges: AtomicUsize,
    }

    unsafe impl Send for CpuDepot {}
    unsafe impl Sync for CpuDepot {}

    impl Default for CpuDepot {
        fn default() -> Self {
            Self::new()
        }
    }

    impl CpuDepot {
        /// Constructs a new per-CPU depot with empty active and backup magazines.
        pub const fn new() -> Self {
            Self {
                active: Magazine::new(),
                backup: Magazine::new(),
                alloc_hits: AtomicUsize::new(0),
                free_hits: AtomicUsize::new(0),
                exchanges: AtomicUsize::new(0),
            }
        }

        /// Allocates an object pointer from the local CPU's magazines.
        ///
        /// 1. Tries active magazine: $O(1)$ pop, zero locks, zero atomics.
        /// 2. If active empty: swaps with backup if backup is not empty.
        /// 3. If both empty: exchanges backup with GlobalDepot for a full magazine.
        /// 4. If depot empty: invokes `refill_fn` to carve fresh objects from slab backend.
        pub fn alloc<F>(&mut self, global_depot: &GlobalDepot, mut refill_fn: F) -> Option<*mut u8>
        where
            F: FnMut(&mut Magazine) -> bool,
        {
            // 1. Fast path: pop from active magazine
            if let Some(ptr) = self.active.pop() {
                self.alloc_hits.fetch_add(1, Ordering::Relaxed);
                return Some(ptr);
            }

            // 2. Active is empty; check backup magazine
            if !self.backup.is_empty() {
                core::mem::swap(&mut self.active, &mut self.backup);
                if let Some(ptr) = self.active.pop() {
                    self.alloc_hits.fetch_add(1, Ordering::Relaxed);
                    return Some(ptr);
                }
            }

            // 3. Both active and backup are empty; exchange backup with global depot
            self.exchanges.fetch_add(1, Ordering::Relaxed);
            if let Some(full_mag) = global_depot.exchange_empty(self.backup) {
                self.backup = full_mag;
                core::mem::swap(&mut self.active, &mut self.backup);
                if let Some(ptr) = self.active.pop() {
                    self.alloc_hits.fetch_add(1, Ordering::Relaxed);
                    return Some(ptr);
                }
            }

            // 4. Depot has no full magazines; refill active magazine via slab backend
            if refill_fn(&mut self.active) {
                self.active.pop()
            } else {
                None
            }
        }

        /// Returns an object pointer back to the local CPU's magazines.
        ///
        /// 1. Tries active magazine: $O(1)$ push, zero locks, zero atomics.
        /// 2. If active full: swaps with backup if backup is not full.
        /// 3. If both full: exchanges backup with GlobalDepot for an empty magazine.
        pub fn free(&mut self, ptr: *mut u8, global_depot: &GlobalDepot) {
            if ptr.is_null() {
                return;
            }

            // 1. Fast path: push to active magazine
            if self.active.push(ptr).is_ok() {
                self.free_hits.fetch_add(1, Ordering::Relaxed);
                return;
            }

            // 2. Active is full; check backup magazine
            if !self.backup.is_full() {
                core::mem::swap(&mut self.active, &mut self.backup);
                if self.active.push(ptr).is_ok() {
                    self.free_hits.fetch_add(1, Ordering::Relaxed);
                    return;
                }
            }

            // 3. Both active and backup are full; exchange backup with global depot
            self.exchanges.fetch_add(1, Ordering::Relaxed);
            if let Some(empty_mag) = global_depot.exchange_full(self.backup) {
                self.backup = empty_mag;
                core::mem::swap(&mut self.active, &mut self.backup);
                let _ = self.active.push(ptr);
            }
        }

        /// Empties both active and backup magazines.
        pub fn drain(&mut self) -> (Magazine, Magazine) {
            let act = self.active;
            let bkp = self.backup;
            self.active.clear();
            self.backup.clear();
            (act, bkp)
        }

        /// Retrieves local allocation hits.
        pub fn alloc_hits(&self) -> usize {
            self.alloc_hits.load(Ordering::Relaxed)
        }

        /// Retrieves local free hits.
        pub fn free_hits(&self) -> usize {
            self.free_hits.load(Ordering::Relaxed)
        }

        /// Retrieves count of depot exchanges.
        pub fn exchanges(&self) -> usize {
            self.exchanges.load(Ordering::Relaxed)
        }

        /// Returns number of objects currently cached in this CPU's active and backup magazines.
        pub fn cached_count(&self) -> usize {
            self.active.rounds() + self.backup.rounds()
        }
    }
}

pub use self::depot::*;
pub use self::percpu::*;
