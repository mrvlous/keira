// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Central Global Depot coordinating magazine exchanges between CPU cores.

use super::Magazine;
use keira_core::sync::{IrqSpinLock, LockRank};

/// Maximum number of full and empty magazines buffered in the central depot.
pub const DEPOT_CAPACITY: usize = 64;

/// Central depot holding pools of full and empty magazines.
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
