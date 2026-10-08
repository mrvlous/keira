// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Per-CPU magazine depot maintaining active and backup magazines.

use super::{GlobalDepot, Magazine};
use core::sync::atomic::{AtomicUsize, Ordering};

/// Maximum symmetric multiprocessing (SMP) CPU cores supported.
pub const MAX_CPU_CORES: usize = 16;

/// Per-CPU magazine pair enabling zero-lock local allocation and deallocation.
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
